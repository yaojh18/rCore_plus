# lab1实验报告
肖泽基 2019011241
# 1.实验内容
* 增加TaskSyscallTimes类，用于记录系统调用次数，并在TaskControlBlock内增加对应成员task\_sys
* 在task模块中增加函数get\_task\_status，用于获得当前程序的运行状态
* 在task模块中增加函数get\_task\_syscall\_times，用于获得当前程序各系统调用次数
* 在task模块中增加函数change\_task\_syscall\_times，用于在发生系统调用时记录其次数
* 在以上修改的基础上，实现了系统调用处理函数sys\_task\_info

# 2.问答题
## 1.
* 对于使用 S 态特权指令的测例ch2b\_bad\_instructions.rs(sret)和ch2b\_bad\_registers.rs(csrr)报错信息如下：

```
[ERROR] [kernel] IllegalInstruction in application, core dumped.
...
[ERROR] [kernel] IllegalInstruction in application, core dumped.
```

* 对于访问错误地址的测例ch2b\_bad\_address.rs报错信息如下：

```
[ERROR] [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x8040008a, core dumped.
```

* 使用的sbi为：RustSBI version 0.2.0-alpha.4

## 2.
* a0代表了函数的传入参数，即TrapContext的存储位置；_restore函数可以在中断、异常处理后或系统调用后用于恢复用户栈并回到用户态
* 改写了sstatus、sepc和sscratch三个csr寄存器，其中改写sstatus恢复了原用户程序操作状态；改写sepc用于之后回到异常或中断发生前的控制流；sscratch用于在下文回到用户栈
* x2是sp寄存器，在下文中通过sscratch寄存器恢复；x4是tp寄存器，这里不会发生改变
* S态到U态状态切换发生在sret之后，该指令使CPU根据sstatus中的SPP字段回到用户态
* 执行指令后，sp为用户栈栈顶地址，sscratch为内核栈栈顶地址
* U态到S态状态切换发生在用户进行系统调用（如ecall）后或异常、中断发生后