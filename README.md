# Labeled RISC-V Rust Bootloader

基于 RustSBI 的 [labeled RISC-V](https://github.com/LvNA-system/labeled-RISC-V/tree/master/fpga) 启动固件，适配 ZCU102 开发板和 [Labeled ucore-SMP](https://github.com/TianhuaTao/uCore-SMP/tree/label-riscv) 。

## 环境和工具配置

将 rust 切换到 nightly ；安装 [just](https://github.com/casey/just#installation) 。

## 编译

```shell
git clone https://github.com/Gallium70/lrv-rust-bl.git
cd lrv-rust-bl
just build
```

## 使用

见 [labeled-RISC-V-boot](https://github.com/Gallium70/labeled-RISC-V-boot)

## 设计

### 内存保护初始化

将 `pmpcfg0` 配置为 `NAPOT | X | W | R` ，将 `pmpaddr0` 全部置 1 ，即允许 S 和 U 态程序在全部地址空间进行读写和执行操作。

将 `satp` 置 0 ，关闭分页。由于该平台使用软启动和复位，故需要显式清除先前程序可能使用过的 CSR 。

### 中断和异常配置

将 S 态的外部、时钟和软件中断，三种页异常和 U 态环境调用委托到 S 态。方便调试起见，没有委托断点异常。非对齐加载和非法指令异常在 M 态处理。

### 指令模拟

在非法指令异常处理中，可以通过访问 RTC 外设模拟 `rdtime` 指令；在非对齐加载/存储异常中，可以通过两次对齐的加载/存储进行模拟，但仅支持 RV64IC 。
### SBI 扩展

#### Legacy Extensions

支持 Set Timer 、Send IPI 、 Console Putchar 和 Console Getchar ，对于 System Shutdown 实现为死循环。

#### Hart State Management Extension

实际上没有状态管理，对于 HART start 只会向相应的 HART 发送一个 IPI ，不会传递参数，这主要是为了以最简单的方式通过 [ucore-SMP](https://github.com/TianhuaTao/uCore-SMP/tree/label-riscv) 的多核启动流程。其他函数会返回 Not Supported 。

