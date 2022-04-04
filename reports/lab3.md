# lab3实验报告
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
* 初始时，执行指令位于0x1000，随后跳转到0x8000\_0000，即rustsbi的start，然后跳转到rustsbi的rust\_main函数
* 在rust\_main中，通过调用delegate\_interrupt\_exception和set\_tmp进行委托终端和权限设置
* 最后，通过execute_supervisor进入0x8020\_0000