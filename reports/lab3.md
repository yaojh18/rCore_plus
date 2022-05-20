# lab3实验报告
肖泽基 2019011241
## 1.实验内容
* 修改了前两个实验的实现，以适应这一章将进程控制块和任务管理器拆分后的数据结构
* 在TaskControlBlock中增加了spawn函数，功能为根据传入的字符串在新进程中执行字符串对应的程序，以满足spawn系统调用
* 在TaskControlBlockInner中增加了stride和priority成员，并在task模块中增加了change\_task\_priority函数来修改priority，以满足set\_priority系统调用
* 修改了TaskManager的成员和add、fetch函数，实现stride调度算法

## 2.问答题
* 不会轮到p1执行。因为p2.stride加pass后本应为260，但是8bit无符号整型最大值为255，发生溢出，实际大小为4，所以下一次也是p2执行
* 原因如下：
	* 对于第i个进程，因为pi.priority>=2，所以pi.pass<=BigStride/2
	* 若当前所有进程第stride满足：STRIDE\_MAX–STRIDE\_MIN<=BigStride/2，设STRIDE\_MAX和STRIDE\_MIN对应的进程为pmax，pmin
	* 下一次调度时，应选择pmin执行，则pmin.stride=STRIDE\_MIN+pass，设此时stride最小的进程为pj（pj仍可能是pmin）
	* 1）若STRIDE\_MIN+pass<=STRIDE\_MAX，则有pmax.stride-pj.stride<=STRIDE\_MAX–STRIDE\_MIN<=BigStride/2
	* 2）若STRIDE\_MIN+pass>STRIDE\_MAX，则有pmin.stride-pj.stride<=STRIDE\_MIN+pass-STRIDE\_MIN=pass<=BigStride/2
	* 根据以上推导，由数学归纳法可以证明：STRIDE\_MAX–STRIDE\_MIN<=BigStride/2
* 代码如下：

```
//...
        if self<other{
            if other.0-self.0<=BIG_STRIDE/2{
                return Some(Ordering::Less);
            }
            return Some(Ordering::Greater);
        }
        if self.0-other.0<=BIG_STRIDE/2{
            return Some(Ordering::Greater);
        }
        return Some(Ordering::Less);
//...
``` 