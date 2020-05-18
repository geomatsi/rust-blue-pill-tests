target remote :3333
set print asm-demangle on
monitor arm semihosting enable

# # make the microcontroller SWO pin output compatible with UART (8N1)
# # 32_000_000 must match the sys clock frequency
# # 2_000_000 is the frequency of the SWO pin

# monitor tpiu config external uart off 32000000 2000000
# monitor itm port 0 on

load
cont
