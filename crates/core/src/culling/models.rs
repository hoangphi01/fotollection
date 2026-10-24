// SPDX-License-Identifier: GPL-3.0-only
//! The bundled face models (ADR-0018): compiled into the binary, checked against pinned
//! BLAKE3 sums, loaded and optimized once per process. Nothing is ever downloaded.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex, OnceLock};

use tract_onnx::prelude::*;

use super::CullingError;

const YUNET: &[u8] = include_bytes!("../../assets/models/yunet.onnx");
const YUNET_BLAKE3: &str = "3d5938c4cd5a02dc416f1cd1f7fc1f662a22adc370477112c871954587e63431";
const FACEMESH: &[u8] = include_bytes!("../../assets/models/facemesh.onnx");
const FACEMESH_BLAKE3: &str = "e01c5f5c50e4dc2020947037d2a08cada1df73a9dc7f14f0c9d464c4ba2c23fa";

/// Long edge of YuNet's input. YuNet takes any size that is a multiple of 32, so the input
/// follows the photo's shape instead of padding it to a square (NFR-014). 512 is an exact
/// halving of the 1024 px analysis image, and a face of 64 analysis pixels, the smallest
/// that gets an eye check, is still 32 px.
pub const YUNET_LONG_EDGE: usize = 512;
/// Square input side of FaceMesh.
pub const FACEMESH_SIDE: usize = 256;

type Plan = Arc<TypedRunnableModel>;
/// YuNet plans by input width and height.
type Plans = Mutex<HashMap<(usize, usize), Arc<Model>>>;

/// A loaded model plus its output names, in output order.
pub struct Model {
    plan: Plan,
    outputs: Vec<String>,
}

impl Model {
    fn load(bytes: &[u8], blake3: &str, shape: [usize; 4]) -> Result<Self, CullingError> {
        if blake3::hash(bytes).to_hex().as_str() != blake3 {
            return Err(CullingError::ModelChecksum);
        }
        // The files record their intermediate shapes for one input size; ignoring them lets
        // YuNet run at the photo's own shape.
        let model = tract_onnx::onnx()
            .with_ignore_value_info(true)
            .with_ignore_output_shapes(true)
            .model_for_read(&mut Cursor::new(bytes))
            .and_then(|m| m.with_input_fact(0, f32::fact(shape).into()))
            .map_err(model_error)?;
        let outputs = model
            .outputs
            .iter()
            .map(|o| {
                model
                    .outlet_label(*o)
                    .unwrap_or(&model.node(o.node).name)
                    .to_owned()
            })
            .collect();
        let plan = model
            .into_optimized()
            .and_then(TypedModel::into_runnable)
            .map_err(model_error)?;
        Ok(Self { plan, outputs })
    }

    /// Runs the model; returns each output's name and values.
    pub fn run(&self, input: Tensor) -> Result<Vec<(&str, Vec<f32>)>, CullingError> {
        let values = self.plan.run(tvec!(input.into())).map_err(model_error)?;
        self.outputs
            .iter()
            .zip(values.iter())
            .map(|(name, v)| {
                let data = v.to_plain_array_view::<f32>().map_err(model_error)?;
                Ok((name.as_str(), data.iter().copied().collect()))
            })
            .collect()
    }
}

fn model_error(e: impl std::fmt::Display) -> CullingError {
    CullingError::Model(e.to_string())
}

fn cached(
    cell: &'static OnceLock<Result<Model, CullingError>>,
    load: impl FnOnce() -> Result<Model, CullingError>,
) -> Result<&'static Model, CullingError> {
    cell.get_or_init(load).as_ref().map_err(Clone::clone)
}

/// The face detector for a `width` × `height` input (multiples of 32), optimized once per
/// shape: a library holds only a few (landscape, portrait, a crop or two).
pub fn yunet(width: usize, height: usize) -> Result<Arc<Model>, CullingError> {
    static PLANS: OnceLock<Plans> = OnceLock::new();
    let plans = PLANS.get_or_init(Mutex::default);
    if let Some(model) = plans
        .lock()
        .map_err(|_| CullingError::Poisoned)?
        .get(&(width, height))
    {
        return Ok(Arc::clone(model));
    }
    // Loaded outside the lock so other shapes and threads are not held up.
    let model = Arc::new(Model::load(YUNET, YUNET_BLAKE3, [1, 3, height, width])?);
    let mut plans = plans.lock().map_err(|_| CullingError::Poisoned)?;
    Ok(Arc::clone(plans.entry((width, height)).or_insert(model)))
}

/// The face landmark model, loaded on first use.
pub fn facemesh() -> Result<&'static Model, CullingError> {
    static CELL: OnceLock<Result<Model, CullingError>> = OnceLock::new();
    cached(&CELL, || {
        Model::load(
            FACEMESH,
            FACEMESH_BLAKE3,
            [1, FACEMESH_SIDE, FACEMESH_SIDE, 3],
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fr_041_bundled_models_load_and_match_their_checksums() {
        let yunet = yunet(512, 384).map_err(|e| e.to_string());
        assert!(yunet.is_ok(), "{yunet:?}", yunet = yunet.err());
        let facemesh = facemesh().map_err(|e| e.to_string());
        assert!(facemesh.is_ok(), "{facemesh:?}", facemesh = facemesh.err());
        assert!(matches!(
            Model::load(YUNET, FACEMESH_BLAKE3, [1, 3, 640, 640]),
            Err(CullingError::ModelChecksum)
        ));
    }
}
