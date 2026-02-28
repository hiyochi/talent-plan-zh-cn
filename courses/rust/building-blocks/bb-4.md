# PNA Rust — 构建模块 4

让我们学习一些构建模块！

把其他项目和杂务放在一边，深呼吸，放松一下。这里有一些有趣的资源供你探索。

请阅读所有材料，完成所有练习，并观看视频。

- **[阅读：Rust 中的无畏并发][f]**。这是来自 [Aaron Turon][at] 的一篇经典 Rust 博客文章，清晰地解释了为什么在 Rust 中并发如此简单。标题也首次使用了“无畏”一词来描述 Rust 的各种特性。

- **[阅读：并发与并行有何区别？][d]**。这是一篇仅需 10 秒阅读的文章，但值得牢记。这两个词常被混用。我们主要使用“并发”一词，因为它比“并行”更通用。有时我们会使用“并行”以更具体地表达，有时则仅仅因为听起来更顺耳……

- **[阅读：Rust：独特的视角][ru]**。由 [Servo] 团队的 [Matt Brubeck][mb] 撰写，解释了可变别名的危险以及 Rust 如何解决这一问题。

- **[视频：Rust 并发详解][ex]**。由 [Alex Crichton][ac] 深入讲解的演讲。Aaron 和 Alex 编写了标准库中许多并发数据结构。Alex 多年来多次做过此演讲，是对 Rust 能力的绝佳概览。

- **[阅读：`std::sync`][ss]**。标准库文档不仅提供了库本身的良好说明，也对相关主题进行了整体概述。本节介绍了标准库提供的大部分并发类型。

- **[练习：基础多线程][bmt]**。这是来自 [rustlings] 项目的一个简单多线程练习。其规模足够小，可以直接在 [play.rust-lang.org] 上完成。

- **练习：编写一个线程池**。

  一个[线程池]会在一组可重用的线程上运行任务（函数），相比为每个任务都新建一个线程，这种方式通常更高效。

  创建一个具有以下类型签名的简单线程池：

  ```rust
  impl ThreadPool {
    fn new(threads: u32) -> Result<Self>;

    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static;
  }
  ```

  `new` 函数应立即创建 `threads` 个线程，这些线程随后将等待任务被提交。当某个线程接收到任务时，它会执行该任务直至完成，然后继续等待下一个任务。

  [`threadpool`][tp1] 库和 Rayon 的 [`ThreadPool`][tp2] 可以为你提供灵感。

- **[阅读：无锁 vs 无等待并发][lf]**。似乎每个人都希望自己的代码是“无锁”的。这到底意味着什么？

<!--

TODO

并发数据结构概览
代码重排序

https://en.wikipedia.org/wiki/Concurrent_data_structure
https://preshing.com/20120625/memory-ordering-at-compile-time/
https://www.cl.cam.ac.uk/~jp622/the_problem_of_programming_language_concurrency_semantics.pdf
并发映射 https://gitlab.nebulanet.cc/xacrimon/rs-hm-bench

-->

[lf]: https://rethinkdb.com/blog/lock-free-vs-wait-free-concurrency/
[play.rust-lang.org]: https://play.rust-lang.org/
[tp1]: https://docs.rs/threadpool/1.7.1/threadpool/struct.ThreadPool.html
[tp2]: https://docs.rs/rayon/1.0.3/rayon/struct.ThreadPool.html
[thread pool]: https://softwareengineering.stackexchange.com/questions/173575/what-is-a-thread-pool#173581
[ss]: https://doc.rust-lang.org/std/sync/index.html
[Servo]: https://github.com/servo/servo
[mb]: https://github.com/mbrubeck/
[ru]: https://limpet.net/mbrubeck/2019/02/07/rust-a-unique-perspective.html
[ac]: https://github.com/alexcrichton/
[ex]: https://www.youtube.com/watch?v=Dbytx0ivH7Q
[f]: https://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html
[d]: https://stackoverflow.com/questions/1050222/what-is-the-difference-between-concurrency-and-parallelism#1050257
[at]: https://github.com/aturon
[bmt]: https://github.com/rust-lang/rustlings/blob/master/exercises/threads/threads1.rs
[rustlings]: https://github.com/rust-lang/rustlings/