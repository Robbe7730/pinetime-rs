target remote :3333

# set backtrace limit 32

# Target config
#set arm force-mode thumb
#monitor arm semihosting enable

# Load binary
load
break main
continue
step
