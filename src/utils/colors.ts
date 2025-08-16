import Aura from '@primeuix/themes/aura'

export const colors = Object.keys(Aura.primitive || {}).filter(
  key => key !== 'borderRadius',
)
