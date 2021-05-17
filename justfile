target := "riscv64imac-unknown-none-elf"
mode := "debug"
build-path := "./target/" + target + "/" + mode + "/"
bootloader-elf := build-path + "lrv-rust-bl"
bootloader-bin := build-path + "lrv-rust-bl.bin"
bootloader-asm := build-path + "lrv-rust-bl.asm"

objdump := "riscv64-unknown-elf-objdump"
objcopy := "riscv64-unknown-elf-objcopy"
gdb := "riscv64-unknown-elf-gdb"

build: bootloader
    @{{objcopy}} -O binary {{bootloader-elf}} {{bootloader-bin}}

bootloader:
    @cargo build --target={{target}}

asm: build
    @{{objdump}} -D -S {{bootloader-elf}} > {{bootloader-asm}}
