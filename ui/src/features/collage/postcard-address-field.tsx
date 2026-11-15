// SPDX-License-Identifier: GPL-3.0-only
import { useTranslation } from 'react-i18next'
import { AddAssist } from '@/components/forms/addassist'
import { commands, type Address, type Country, type PhotoId } from '@/ipc/bindings'
import { useLiveQuery } from '@/features/settings/use-ipc-query'

const load = async (subject: string) => {
  const result = await commands.getPostcardText(Number(subject))
  return result.status === 'ok' ? result.data : null
}

type PostcardAddressFieldProps = {
  photo: PhotoId
  country: Country
  revision: number
  assisted: boolean
  onAssisted: (assisted: boolean) => void
}

/**
 * The recipient of the chosen Japanese or Korean card (FR-130, FR-132): AddAssist's block,
 * saved on the photo as each field is left (one undoable step, checked in Rust).
 */
export function PostcardAddressField({ photo, country, revision, assisted, onAssisted }: PostcardAddressFieldProps) {
  const { t } = useTranslation()
  // The last answer holds its place while the next loads (LESSONS).
  // This card's own record: the other country's and the European address are never read.
  const recipient = useLiveQuery(String(photo), revision, load).data?.recipients?.find((r) => r.country === country) ?? null
  const save = (address: Address) => void commands.setPostcardAddress(photo, country, address)
  return (
    <AddAssist
      key={photo}
      label={t('addassist.recipient')}
      country={country}
      value={recipient}
      onCommit={save}
      assisted={assisted}
      onAssisted={onAssisted}
    />
  )
}
