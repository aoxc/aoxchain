# CAPABILITY MODEL

AOXCVM’de yetki, sözleşme içi role-map yaklaşımına bırakılmaz; first-class capability object ile temsil edilir.

## Capability examples

- transfer
- mint
- burn
- deploy
- publish/upgrade
- vault spend
- governance mutate

Her kritik mutation için açık capability zorunludur; yoksa yürütme policy aşamasında reddedilir.
