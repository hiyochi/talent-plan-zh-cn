# PNA Rust 项目 4：并发与并行

**任务**：创建一个**多线程**、持久化的键值存储服务器和客户端，使用自定义协议通过同步网络进行通信。

**目标**：

- 编写一个简单的线程池
- 使用通道进行跨线程通信
- 使用锁共享数据结构
- 在不加锁的情况下执行读操作
- 对单线程与多线程进行基准测试

**主题**：线程池、通道、锁、无锁数据结构、原子操作、参数化基准测试。

- [引言](#user-content-introduction)
- [项目规范](#user-content-project-spec)
- [项目设置](#user-content-project-setup)
- [背景：阻塞与多线程](#user-content-background-blocking-and-multithreading)
- [第一部分：多线程](#user-content-part-1-multithreading)
- [第二部分：创建共享的 `KvsEngine`](#user-content-part-2-creating-a-shared-kvsengine)
- [第三部分：为 `KvServer` 添加多线程](#user-content-part-3-adding-multithreading-to-kvserver)
- [第四部分：创建真正的线程池](#user-content-part-4-creating-a-real-thread-pool)
  - [那么，如何构建一个线程池？](#user-content-so-how-do-you-build-a-thread-pool)
- [第五部分：抽象化的线程池](#user-content-part-5-abstracted-thread-pools)
- [第六部分：评估你的线程池](#user-content-part-6-evaluating-your-thread-pool)
  - [好吧，现在开始前两个基准测试](#user-content-ok-now-to-the-first-two-benchmarks)
- [第七部分：评估其他线程池和引擎](#user-content-part-7-evaluating-other-thread-pools-and-engines)
  - [扩展 1：比较函数](#user-content-extension-1-comparing-functions)
  - [背景：锁的局限性](#user-content-background-the-limits-of-locks)
- [第八部分：无锁读取器](#user-content-part-8-lock-free-readers)
  - [解释我们的示例数据结构](#user-content-explaining-our-example-data-structure)
  - [拆分锁的策略](#user-content-strategies-for-breaking-up-locks)
    - [理解并维护顺序一致性](#user-content-understand-and-maintain-sequential-consistency)
    - [识别不可变值](#user-content-identify-immutable-values)
    - [复制值而非共享](#user-content-duplicate-values-instead-of-sharing)
    - [按角色拆分数据结构](#user-content-break-up-data-structures-by-role)
    - [使用专用的并发数据结构](#user-content-use-specialized-concurrent-data-structures)
    - [推迟清理到稍后](#user-content-postpone-cleanup-until-later)
    - [使用原子操作共享标志和计数器](#user-content-share-flags-and-counters-with-atomics)
  - [实现无锁读取器](#user-content-implement-lock-free-readers)


## 引言

在本项目中，你将创建一个简单的键值服务器和客户端，它们通过自定义协议进行通信。服务器将使用同步网络，并使用越来越复杂的并发实现来响应多个请求。内存索引将成为一个由所有线程共享的并发数据结构，压缩操作将在一个专用线程上执行，以减少单个请求的延迟。

## 项目规范

cargo 项目 `kvs` 构建了一个命令行键值存储客户端 `kvs-client` 和一个键值存储服务器 `kvs-server`，两者都调用一个名为 `kvs` 的库。客户端通过自定义协议与服务器通信。

CLI 接口与[上一个项目]相同。这次的区别在于并发实现，我们将逐步介绍它。

[上一个项目]: ../project-3/README.md

库接口几乎相同，但有两个变化。首先，这次所有 `KvsEngine`、`KvStore` 等方法都采用 `&self` 而不是之前的 `&mut self`，并且现在实现了 `Clone`。这在并发数据结构中很常见。为什么？这并不是说我们不会编写不可变代码。而是它将被多个线程共享。为什么这会阻止我们在方法签名中使用 `&mut self`？如果你现在还不清楚，到本项目结束时就会显而易见了。

其次，本项目中的库包含一个新的**特征**，`ThreadPool`。它包含以下方法：

- `ThreadPool::new(threads: u32) -> Result<ThreadPool>`

  创建一个新的线程池，立即启动指定数量的线程。

  如果任何线程启动失败，则返回错误。所有已启动的线程都将被终止。

- `ThreadPool::spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static`

  在线程池中启动一个函数。

  启动总是成功，但如果函数发生恐慌，线程池仍将继续运行，线程数量不会减少，线程池也不会被破坏或失效。

在本项目结束时，将有几种该特征的实现，你将再次进行基准测试以比较它们。

本项目不需要对客户端代码做任何更改。

## 项目设置

继续使用你之前的项目，删除之前的 `tests` 目录，并将本项目的 `tests` 目录复制到其位置。本项目应包含一个名为 `kvs` 的库，以及两个可执行文件 `kvs-server` 和 `kvs-client`。

你需要在 `Cargo.toml` 中添加以下开发依赖项：

```toml
[dev-dependencies]
assert_cmd = "0.11"
criterion = "0.2.11"
crossbeam-utils = "0.6.5"
predicates = "1.0.0"
rand = "0.6.5"
tempfile = "3.0.7"
walkdir = "2.2.7"
panic-control = "0.1.4"
```

与之前的项目一样，添加足够的定义，使测试套件能够构建。

## 背景：阻塞与多线程

到目前为止，你一直在单个线程上处理所有请求，无论是读取还是写入（例如“get”和“set”）。换句话说，数据库中的所有请求都是**序列化**的。使用我们将在本项目中重复使用的图表，时间流如下所示：

```
    thread
           +  +--------+--------+--------+--------+
      T1   |  |   R1   |   R2   |   W1   |   W2   |
           +  +--------+--------+--------+--------+

              --> read/write reqs over time -->
```

读取和写入操作都可能需要**阻塞**。阻塞是指线程在等待访问资源（如来自文件的数据或受锁保护的变量）时停止执行。当一个线程因一个任务被阻塞时，它无法在另一个任务上取得进展。因此，在 I/O 密集型系统中，任何特定请求可能大部分时间都在等待操作系统和内存控制器在磁盘之间移动数据：

```
          +---------+----------------------------+---------+
      R1  | working | waiting for data ...       | working |
          +---------+----------------------------+---------+

          --> time -->
```

在请求被阻塞时，让 CPU 重新投入工作的最简单方法是使用多个线程来处理请求，这样理想情况下我们的请求都能并发处理，如果拥有足够的 CPU，甚至可以并行处理：

```
    thread
           +  +--------+
      T1   |  |   R1   |
           |  +--------+
      T2   |  |   R2   |
           |  +--------+
      T3   |  |   W1   |
           |  +--------+
      T4   |  |   W2   |
           +  +--------+

              --> read/write reqs over time -->
```

因此，本项目的主要目标是并行处理请求。

## 第一部分：多线程

你引入并发的第一个尝试将是最简单的：为每个传入连接创建一个新线程，并在该连接上响应请求，然后让线程退出。跨线程分配工作会带来哪些性能优势？你预计延迟会如何变化？吞吐量呢？

第一步是为这种简单方法编写一个 `ThreadPool` 实现，其中 `ThreadPool::spawn` 将为每个启动的任务创建一个新线程。将其称为 `NaiveThreadPool`（它甚至不是真正的线程“池”，因为此实现不会在任务之间重用线程，但它需要符合我们稍后比较的特征）。

我们现在不关注更复杂的实现，因为仅将此解决方案集成到我们现有的设计中就需要一些努力。注意 `ThreadPool::new` 构造函数接受一个 `threads` 参数，指定池中的线程数。在此实现中，它将被忽略。

**现在就去实现这个版本的 `ThreadPool`**，然后我们将它集成到新的 `KvStore` 中。

**需要完成的测试用例**：

  - `thread_pool::naive_thread_pool_*`


## 第二部分：创建共享的 `KvsEngine`

在将 `NaiveThreadPool` 集成到 `KvServer` 之前，我们必须使 `KvsEngine` 特征和 `KvStore` 实现（目前你可以忽略上一个项目中的 `SledKvsEngine`，但你可以选择性地将其作为本项目的扩展重新实现）。

回想项目规范，这次我们的 `KvsEngine` 将 `self` 作为 `&self` 而不是像以前那样作为 `&mut self`，并且它实现了 `Clone`，这必须为每个实现显式完成，以及 `Send + 'static`，这是每个实现定义的隐式属性。更具体地说，它看起来像：

```rust
pub trait KvsEngine: Clone + Send + 'static {
    fn set(&self, key: String, value: String) -> Result<()>;

    fn get(&self, key: String) -> Result<Option<String>>;

    fn remove(&self, key: String) -> Result<()>;
}
```

这给了我们很多关于我们要追求的实现策略的线索。首先，思考为什么在多线程实现中引擎需要实现 `Clone`。考虑 Rust 中其他并发数据类型的结构，比如 [`Arc`]。现在思考为什么这使我们使用 `&self` 而不是 `&mut self`。关于共享可变状态，你知道什么？到本项目结束时，一定要理解这里的含义——_这就是 Rust 的全部意义所在_。

[`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html

在这种模型中，`KvsEngine` 表现为另一个对象的**句柄**，由于该对象在多个线程之间共享，它可能需要位于[堆]上，并且由于共享状态不能是可变的，它需要被某种同步原语保护。

[堆]: https://stackoverflow.com/questions/79923/what-and-where-are-the-stack-and-heap

因此，_将你的 `KvsEngine` 实现（`KvStore`）内部的数据移动到堆上，使用线程安全的共享指针类型，并用你选择的锁保护它_。

由于 `SledKvsEngine` 实现了 `KvsEngine`，它也可能需要更改。

此时，你的单线程 `kvs-server` 应该再次正常工作，但现在使用一个可以稍后跨线程共享的 `KvsEngine`。

**需要完成的测试用例**：

  - `kv_store::concurrent_*`


## 第三部分：为 `KvServer` 添加多线程

让我们快速回顾一下我们的架构：`KvServer` 设置一个 TCP 套接字并开始监听；当收到请求时，它反序列化请求并调用某个 `KvsEngine` 特征的实现来存储或检索数据；然后发送响应。`KvsEngine` 的工作细节对 `KvServer` 来说无关紧要。

因此，在上一个项目中，你可能创建了一个类似这样的循环：

```rust
let listener = TcpListener::bind(addr)?;

for stream in listener.incoming() {
	let cmd = self.read_cmd(&stream);
	let resp = self.process_cmd(cmd);
	self.respond(&stream, resp);
}
```

_现在你只需要做同样的事情，但在循环内将所有工作交给你的 `NaiveThreadPool`_。数据库查询和响应都在与 TCP 监听器不同的线程上处理。这将大部分繁重的工作卸载到其他线程，使接收线程能够处理更多请求。这应该会增加吞吐量，至少在多核机器上如此。

同样，你现在应该有一个正常工作的客户端/服务器键值存储，现在是多线程的。

## 第四部分：创建真正的线程池

现在你已经建立了多线程架构，是时候编写一个真正的线程池了。在实践中你可能不会自己编写线程池，因为存在经过良好测试的线程池 crate，但这是一个有用的练习，可以让你获得一般的并发经验。稍后在本项目中，我们将像上一个项目中的引擎一样抽象线程池，并将你的性能与现有实现进行比较。

那么，什么是线程池？

它并不复杂。与其为每个多线程任务创建一个新线程，线程池维护一个“线程池”，并重用这些线程而不是创建新的。

但为什么？

这完全是为了性能。重用线程节省了一点性能，但在编写高性能应用程序时，每一点都很重要。想象一下创建一个新线程需要什么：

你需要一个线程运行的调用栈。该调用栈必须被分配。分配相当便宜，但不如不分配便宜。调用栈的分配方式取决于操作系统和运行时的细节，但可能涉及锁和系统调用。系统调用本身并不“那么昂贵”，但当我们处理 Rust 级别的性能时，它们是昂贵的——减少系统调用是常见的简单优化来源。然后该栈必须仔细初始化，以便第一个[栈帧]包含基指针和栈初始[函数序言][fp]所需的任何其他值。在 Rust 中，栈需要配置一个[保护页]以防止栈溢出，从而保持内存安全。这需要两个额外的系统调用，[对 `mmap` 和 `mprotect`][mp]（尽管在 Linux 上，这两个系统调用通常被避免）。

[保护页]: https://docs.microsoft.com/en-us/windows/desktop/Memory/creating-guard-pages
[fp]: https://en.wikipedia.org/wiki/Function_prologue
[栈帧]: https://en.wikipedia.org/wiki/Stack_frame
[2mb]: https://github.com/rust-lang/rust/blob/6635fbed4ca8c65822f99e994735bd1877fb063e/src/libstd/sys/unix/thread.rs#L12
[mp]: https://github.com/rust-lang/rust/blob/6635fbed4ca8c65822f99e994735bd1877fb063e/src/libstd/sys/unix/thread.rs#L315

<!-- TODO: illustration? -->

这只是设置调用栈。至少还需要另一个系统调用来创建新线程，此时内核必须为其自己的内部计数进行新线程的记录。

在 Rust 中，C [libpthread] 库处理了大部分复杂性。

然后在某个时刻，操作系统执行一个[上下文切换]到新栈，线程开始运行。当线程终止时，所有这些工作都需要再次撤销。

使用线程池，所有这些设置开销只对少数线程执行一次，后续任务只是上下文切换到池中的现有线程。

[libpthread]: https://www.gnu.org/software/hurd/libpthread.html
[上下文切换]: https://en.wikipedia.org/wiki/Context_switch


### 那么，如何构建一个线程池？

有许多策略和权衡，但在此练习中，你将使用一个共享队列将工作分发给空闲线程。这意味着你的“生产者”，即接受网络连接的线程，将任务发送到一个单一队列（或通道），而“消费者”，即池中的每个空闲线程，从该通道读取等待执行的任务。这是最简单的任务调度策略，但可能有效。缺点是什么？

你有三个重要的考虑因素：

1) _使用哪种数据结构来分发工作_ —— 它将是一个队列，将有一个发送者（“生产者”），即监听 TCP 连接的线程，以及许多接收者（“消费者”），即池中的线程。

2) _如何处理恐慌任务_ —— 你的池运行任意工作项。如果一个线程恐慌，线程池需要以某种方式恢复。

3) _如何处理关闭_ —— 当 `ThreadPool` 对象超出作用域时，它需要关闭每个线程。它不能让它们空闲。

这些关注点都是相互关联的，因为处理每个问题都可能涉及线程之间的通信和同步。一些解决方案将很简单，每个问题的解决方案能优雅地协同工作；一些解决方案将很复杂，每个问题的解决方案独立且繁琐。仔细选择你的数据结构，并明智地利用它们的功能。

你将通过在某种并发队列类型上发送消息来分发工作（Rust 中的并发队列通常是具有两个连接类型的数据结构：发送者类型和接收者类型；并且可以在两者之间发送任何实现 `Send` + `'static` 的类型）。

Rust 中的消息通常表示为枚举，每个可能发送的消息都有变体，例如：

```rust
enum ThreadPoolMessage {
    RunJob(Box<dyn FnOnce() + Send + 'static>),
    Shutdown,
}
```

这往往比试图“同时处理”多个用于不同目的的通道更简单、更高效。当然，如果只有一种类型的消息，则不需要枚举。现在，上面的例子可能或可能不是管理线程池所需的完整消息集，这取决于设计。特别是，关闭通常可以在队列返回一个指示发送者已被销毁的结果时隐式完成。

Rust 中有许多类型的多线程队列。最常见的是 [`mpsc`] 通道，因为它存在于 Rust 的标准库中。这是一个多生产者、单消费者队列，因此将其用于你的单队列线程池将需要某种锁。在这里使用锁的缺点是什么？Rust 中还有许多其他并发队列类型，每种都有其优缺点。如果你愿意在生产者和消费者端都使用锁，那么你甚至可以使用 `Mutex<VecDeque>`，但在生产环境中，当有更好的解决方案时，可能没有理由这样做。

[`mpsc`]: https://doc.rust-lang.org/std/sync/mpsc/index.html

_历史注释：Rust 标准库中通道的存在有点奇怪，一些人认为这是一个错误，因为它违背了 Rust 保持标准库最小化、专注于抽象操作系统并让 crate 生态系统实验高级数据结构的一般哲学。它们的存在是 Rust 开发历史和作为类似 Go 的消息传递语言起源的产物。其他库如 [`crossbeam`] 提供了更复杂的替代方案，有时更合适的选择_ 😉。

[`crossbeam`]: https://github.com/crossbeam-rs/crossbeam

你的线程池需要处理已启动函数恐慌的情况——简单地让恐慌摧毁池中的线程会迅速耗尽其可用线程。因此，如果池中的线程发生恐慌，你需要确保线程总数不会减少。那么你应该怎么做？你至少有两个选择：让线程死亡并启动另一个，或捕获恐慌并保持现有线程运行。权衡是什么？你必须选择一个，但在代码中留下注释解释你的选择。

你可用的一些工具包括 [`thread::spawn`]、[`thread::panicking`]、[`catch_unwind`]、[`mpsc`] 通道、[`Mutex`]、[crossbeam 的 MPMC 通道][mpmc] 和 `thread` 的 [`JoinHandle`]。你可以使用其中任何一个，但可能不会全部使用。

[`thread::spawn`]: https://doc.rust-lang.org/std/thread/fn.spawn.html
[`thread::panicking`]: https://doc.rust-lang.org/std/thread/fn.panicking.html
[`catch_unwind`]: https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
[`mpsc`]: https://doc.rust-lang.org/std/sync/mpsc/index.html
[`Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
[mpmc]: https://docs.rs/crossbeam/0.7.1/crossbeam/channel/index.html
[`JoinHandle`]: https://doc.rust-lang.org/std/thread/struct.JoinHandle.html

_创建 `SharedQueueThreadPool` 类型，实现 `ThreadPool`_。

**需要完成的测试用例**：

  - `shared_queue_thread_pool_*`

将 `KvServer` 中使用的 `NaiveThreadPool` 替换为 `SharedQueueThreadPool`。同样，你的 `kvs-server` 应该像以前一样工作，但现在有一个稍微更聪明的多线程模型。这次你希望使用适当数量的线程调用线程池构造函数。现在你可以为每个 CPU 创建一个线程，使用 [`num_cpus`] crate。我们稍后会重新审视线程数量。

[`num_cpus`]: https://docs.rs/num_cpus/


## 第五部分：抽象化的线程池

就像在上一个项目中你创建了 `KvsEngine` 抽象来比较不同实现一样，现在你将使用 `ThreadPool` 抽象来做同样的事情。

如果你还没有这样做，请为 `KvServer` 添加第二个类型参数以表示 `ThreadPool` 实现，构造函数接受线程池作为第二个参数，并使用该线程池分发工作。

最后创建另一个 `ThreadPool` 实现，`RayonThreadPool`，使用 [`rayon`] crate 中的 `ThreadPool` 类型。

Rayon 的线程池使用一种更复杂的调度策略，称为["工作窃取"][ws]，我们预计它会比我们的更好，但谁知道呢，直到我们尝试！

[`rayon`]: https://docs.rs/rayon/
[ws]: https://www.dre.vanderbilt.edu/~schmidt/PDF/work-stealing-dequeue.pdf


## 第六部分：评估你的线程池

现在你将编写**六个**基准测试，一个写密集型工作负载，比较 `SharedQueueThreadPool` 在不同线程数下的性能，一个读密集型工作负载，比较 `SharedQueueThreadPool` 在不同线程数下的性能；另外两个使用 `RayonThreadPool` 代替 `SharedQueueThreadPool`，最后两个使用 `RayonThreadPool` 结合 `SledKvsEngine`。

这并不像听起来那么繁重——其中四个基本上是前两个的重复。

_注意：接下来的两个部分描述了一组相当复杂的基准测试。它们可以被编写（可能……还没有人做过），但理解和高效编写可能具有挑战性。这些部分介绍了一些有用的 criterion 特性，但如果太压倒性，跳过[向前]（并提交一个关于什么不起作用的错误）是可以的。另一方面，这里的难度可能是一个很好的学习机会。最后，按所述实现这些基准测试需要一种以编程方式关闭 `KvsServer` 的方法（即不发送 `SIGKILL` 让操作系统去做），我们之前没有讨论过这一点。_

[向前]: #user-content-background-the-limit-of-locks

因此，作为其中的一部分，你需要确保你在上一个项目中编写的 `SledKvsEngine` 实现在这个多线程上下文中再次工作。这应该是微不足道的，因为 sled 可以被克隆并在线程之间发送，就像你的引擎一样。

希望结果会很有趣。

再次，你将使用 criterion。

这些将是**参数化**基准测试，即单个基准测试多次运行，使用不同的参数。Criterion 将其称为[使用输入进行基准测试][bi]。你基准测试的参数将是线程池中的线程数。

你试图测试的是你的服务器在各种条件下的吞吐量。你将并发发送许多请求，等待响应，然后结束。你在这里应该好奇的是线程数如何影响你的吞吐量，与你的机器上的 CPU 数量相比；你的线程池与 rayon 的比较；以及你的 `KvStore` 在多线程上下文中与 `SledKvsEngine` 的比较。

这将因你的 `KvsClient` 是（可能）阻塞的而变得复杂，也就是说，它发送一个请求然后等待响应。如果它是非阻塞的，那么你可以发送许多请求而不等待响应，然后稍后收集响应。使用阻塞的 `KvsClient`，你将需要在每个线程中发送每个请求，以饱和服务器的容量。

在基准测试时，重要的是要确切了解你试图测量的代码，并尽可能只测量该代码。像 criterion 这样的基准测试工具在一个循环中多次运行一段代码，测量每次循环所花费的时间。因此，我们希望只将我们想要测量的代码放在循环中，尽可能多地将其他内容留在循环之外。

[bi]: https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_with_inputs.html

所以以这个 criterion 基准测试带有输入的简单例子为例：

```rust
let c = Criterion::default();
let inputs = &[1, 2, 3, 4, 5];

c.bench_function_over_inputs("example", |b, &&num| {
    b.iter(|| {
        // 重要的测量工作放在这里
	});
}, inputs);
```

`iter` 多次调用你的闭包，测量每次迭代。但由于你将需要提前设置大量线程，这是你不希望测量的工作。如果你可以为多次迭代只设置一次，那么设置可以放在闭包之外，像这样：

```rust
let c = Criterion::default();
let inputs = &[1, 2, 3, 4, 5];

c.bench_function_over_inputs("example", |b, &&num| {
    // 在这里进行设置
    b.iter(|| {
        // 重要的测量工作放在这里
	});
}, inputs);
```

`b.iter` 闭包内的代码是被测量的，设置在之前。

如果设置不能在循环之前进行，那么另一种策略是使设置工作量小于你实际想测量的工作量，例如通过添加循环。还要考虑基准测试的“清理”，这通常主要由运行 `drop` 实现组成，也有成本。

如果你有一个阻塞客户端，你将需要许多客户端线程，而你只有一次机会在循环的多次迭代之前创建这些线程。因此，你需要在迭代之前设置一堆可重用的线程。幸运的是，你有完美的工具来做到这一点，即你的 `SharedQueueThreadPool`。用每个请求一个线程来设置它，并配以一些通道来报告响应已收到，你将拥有一个合适的基准测试框架。

### 好吧，现在开始前两个基准测试

我们说过这是一个参数化基准测试，基准测试的参数是服务器线程池中使用的 CPU 数量。我们想看看使用 1 个线程、2 个线程、4 个线程，然后是每两个线程直到你计算机 CPU 数量的两倍时的吞吐量如何。为什么是 2 倍？因为可能有比核心数更多的线程的好处，你将通过实验发现。

对于写密集型工作负载，在设置阶段（在调用 `b.iter(...)` 之前），创建 `KvServer<KvStore, SharedQueueThreadPool>`，线程池包含参数化数量的线程。然后编写一个工作负载，设置 1000 个唯一键，长度相同，全部设置为相同的值。注意，虽然键不同，但为了获得一致的结果，每次基准测试循环都需要相同的键。

然后，每个线程设置一个值后，还应 `assert!` 调用成功（以确保在负载下没有错误），然后表示已完成。当所有线程完成后，基准测试线程继续并完成该次迭代。实现这种完成信号的明显方法是每个线程向基准测试线程发送一条消息，但请记住，信号代码是与你试图测量的代码无关的开销，因此它需要做最少的工作。你能只用一条消息，或者也许用其他只向基准测试线程发送一次信号的并发类型来完成吗？

将此基准测试命名为 `write_queued_kvstore`（或任何名称）。

对于读密集型工作负载，在设置阶段，创建 `KvServer<KvStore, SharedQueueThreadPool>`，线程池包含参数化数量的线程，并创建包含 1000 个线程的客户端线程池。仍在设置阶段，再创建一个客户端并初始化 1000 个唯一键，长度相同，全部设置为相同的值。

然后，在基准测试循环期间，从客户端，启动 1000 个任务来检索这些相同的键/值对，然后 `assert!` 结果正确。最后，像之前一样，向基准测试线程发送一条消息，表示读取完成。

将此基准测试命名为 `read_queued_kvstore`（或任何名称）。

**哇。这工作量真大**。

因此，你可以像往常一样使用 `cargo bench` 运行这一组 criterion 基准测试。

<!-- TODO show example results -->

但这次你将做更多。由于你对多个参数运行相同的基准测试，代表线程池中的线程数，如果我们能以漂亮的图表看到不同线程数的影响，那就太好了。

哦，嘿——criterion 就能做到！

回去阅读关于[使用输入进行基准测试][bi]的内容。它解释了如何查看基准测试与其输入的图表。你注意到了什么？当你线程数接近机器上的 CPU 数量时会发生什么？当线程数超过机器上的线程数时会发生什么？你认为是什么导致了你看到的趋势？结果取决于许多因素，因此你的结果可能与任何其他人的不同。

这是始终进行基准测试而不是对性能进行推测的好理由。我们可以做出有根据的猜测，但直到我们测试才知道。

<!-- TODO: not sure if this would actually improve perf

## 扩展 1：千线程方法的替代方案

如上所述，为了编写你的基准测试，你需要启动 1000 个线程，每个客户端生成对服务器的负载。这是必要的，因为 `KvsClient` 的 `get` 和 `set` 方法_阻塞_等待操作的结果。这将导致大量开销，很可能影响你的基准测试质量。这里的开销不是来自启动和销毁线程，因为你已通过你的 `ThreadPool` 设置将这些工作放在基准测试循环之外。_但是_，因为每个请求都在不同的线程上生成，这意味着每个请求都需要在这些线程被调度时进入和退出内核的上下文切换。

如果单个线程可以一次发出许多请求，然后稍后等待它们的结果，那就更好了。

这是简单的_异步_编程风格，这不是本项目的话题，但将是下一个项目的话题。

不过，对于这个项目，如果你想创建一个更高效的基准测试，有一个简单的方法，通过让“set”方法返回一个稍后可以等待的句柄。

因此，想象一下你的 `KvsClient` API 如今看起来像：

``rust
pub fn get(&mut self, key: String) -> Result<Option<String>>;
pub fn set(&mut self, key: String, value: String) -> Result<()>;
pub fn remove(&mut self, key: String) -> Result<()>;
```

如果你改为添加一组新方法：

``rust
pub fn get_async(&mut self, key: String) -> Result<QueryHandle>;
pub fn set_async(&mut self, key: String, value: String) -> Result<QueryHandle>;
pub fn remove_async(&mut self, key: String) -> Result<QueryHandle>;
pub fn wait_for_result(&mut self, q: QueryHandle) -> Result<QueryResult>;
```

那么你可以，例如，一次发出许多查询，将句柄存储在向量中，每个句柄包含一个打开的 TCP 流，然后稍后依次等待每个结果。这将使你的基准测试客户端线程池包含少得多的线程（可能每个 CPU 一个线程即可）。

我们不会在这里深入探讨这个解决方案，但你可能想在这个方向上进行实验，特别是如果你发现你的基准测试比较不有趣的话。

TODO: 我们能解释如何使用 perf 来测量上下文切换时间吗？

-->


## 第七部分：评估其他线程池和引擎

好的。你已经完成了这个基准测试练习中最困难的部分。现在你只需要在几种配置中做几乎相同的事情。

取你之前编写的两个基准测试，将它们复制粘贴三次。在所有这些中，将 `SharedQueueThreadPool` 更改为 `RayonThreadPool`。

第三和第四个，命名为 `read/write_rayon_kvstore`（或任何名称）。这些你将与前两个 `SharedQueueThreadPool` 实现进行比较，以查看你的实现与 `RayonThreadPool` 之间的差异。

第四和第五个，命名为 `read/write_rayon_sledkvengine`，并将引擎更改为 `SledKvsEngine`。这些你将与前两个进行比较，以查看你的 `KvsEngine` 在多线程环境中与 sled 的比较。

和之前一样，运行并绘制所有这些基准测试。如上所述，将它们相互比较。在各种线程数下，你的调度器与 rayon 相比如何？在各种线程数下，你的存储引擎与 sled 相比如何？结果令人惊讶吗？你能想象为什么存在这些差异吗？

<!-- 现在将是_绝佳_时间阅读 [rayon 的源码] 和 [sled 的源码]。习惯阅读别人的源码。那是你学到最多的地方。 -->


### 扩展 1：比较函数

现在你有了三个不同线程池的相同基准测试，并且你已经运行并比较了它们的性能。Criterion 内置了比较多个实现的支持。查看 Criterion 用户指南中的["比较函数"][cp]，修改你的基准测试，让 criterion 自己进行比较。查看那些精美的图表。

[cp]: https://bheisler.github.io/criterion.rs/book/user_guide/comparing_functions.html


### 背景：锁的局限性

在本项目早期，我们建议通过将 `KvsEngine` 的内部放在堆上并用锁保护来使其线程安全。你可能立即意识到这不会提高吞吐量，因为它用一种阻塞交换了另一种——不是可能阻塞在磁盘访问上，而是现在肯定阻塞在互斥锁访问上。

因此，到目前为止我们所实现的是：

```
    thread
           +  +--------+
      T1   |  |   R1   |
           |  +-----------------+
      T2   |           |   R2   |
           |           +-----------------+
      T3   |                    |   W1   |
           |                    +-----------------+
      T4   |                             |   W2   |
           +                             +--------+
              --> read/write reqs over time -->
```

在上一节中，你对你的引擎与 `SledKvsEngine` 的多线程吞吐量进行了基准测试。希望你发现你的多线程实现比 `sled` 的多线程实现表现得差得多（如果不是，那么你超级棒或者 `sled` 有一些问题）。到目前为止，添加多线程导致性能明显比单线程实现更差——现在你有了线程间上下文切换的额外工作，以及互斥锁带来的保证阻塞。

因此，对于本项目这部分，你将变得更加复杂。用锁保护整个状态很容易——整个状态总是原子地读取和写入，因为一次只有一个客户端可以访问整个状态。但这意味着两个想要访问共享状态的线程必须互相等待。换句话说，当 `KvsEngine` 受互斥锁保护时，尽管是多线程的，服务器中实际的并发性非常少。

高性能、可扩展的并行软件倾向于尽可能避免锁和锁竞争。Rust 使复杂的高性能并发模式比大多数语言更容易（因为你不需要担心数据竞争和崩溃），但它_不会_保护你免受导致错误行为的逻辑错误。

因此，你仍然必须对并发进行一些艰难的思考。幸运的是，Rust crate 生态系统中有许多复杂的并行编程工具，因此你的任务通常是理解它们是什么以及如何将它们组合起来，而不是理解如何编写复杂的无锁数据结构。

让我们看一些逐步更复杂的例子。我们将以一个单线程 `KvStore` 为例，考虑如何使其线程安全。

这是一个像你可能在早期项目中创建的单线程 `KvStore` 示例（这是课程示例项目中的简化版本）：

```rust
pub struct KvStore {
    /// 日志和其他数据的目录
    path: PathBuf,
    /// 日志读取器
    reader: BufReaderWithPos<File>,
    /// 日志写入器
    writer: BufWriterWithPos<File>,
    /// 从键到日志指针的内存索引
    index: BTreeMap<String, CommandPos>,
    /// 表示在压缩期间可以删除的“陈旧”命令的字节数
    uncompacted: u64,
}
```

这是简单的多线程版本，用锁保护一切。希望你为本项目已经写的内容看起来像这样：

```rust
#[derive(Clone)]
pub struct KvStore(Arc<Mutex<SharedKvStore>>);

#[derive(Clone)]
pub struct SharedKvStore {
    /// 日志和其他数据的目录
    path: PathBuf,
    /// 日志读取器
    reader: BufReaderWithPos<File>,
    /// 日志写入器
    writer: BufWriterWithPos<File>,
    /// 从键到日志指针的内存索引
    index: BTreeMap<String, CommandPos>,
    /// 表示在压缩期间可以删除的“陈旧”命令的字节数
    uncompacted: u64,
}
```

这个 `Arc<Mutex<T>>` 解决方案是简单、正确且常见的：

- [`Arc`] 将值放在堆上，以便在线程之间共享，并提供一个 `clone` 方法为每个线程创建一个“句柄”。
- [`Mutex`] 提供了一种在没有现有 `&mut` 引用的情况下重新获得写访问权限的方法。

[`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
[`Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html

这在许多情况下都是一个合理的解决方案。但在此情况下，该互斥锁在负载下将成为_竞争_的来源：`Mutex` 不仅序列化对 `SharedKvStore` 的写访问，还序列化读访问。任何想要使用 `KvStore` 的线程都需要等待另一个线程释放 `Mutex`。任何请求都会阻塞任何其他并发请求。

我们_真正_想要的是不获取锁，或者——如果必须使用锁——让它们很少与其他线程竞争。

比 `Mutex` 更进一步的复杂性是 [`RwLock`]，即“读写锁”。这是每个并行软件程序员都必须知道的另一种常见锁类型。读写锁相对于互斥锁的改进是它允许_任意数量的读者_，或_单个写者_。因此，在 Rust 术语中，`RwLock` 将同时分发任意数量的 `&` 指针，或单个 `&mut` 指针。读者仍然阻塞写者，写者仍然阻塞读者和其他写者。

[`RwLock`]: https://doc.rust-lang.org/std/sync/struct.RwLock.html

在我们的数据库中，这意味着所有读请求可以并发满足，但当单个写请求进来时，系统中的所有其他活动都会停止并等待它。实现这一点基本上只是将 `Mutex` 替换为 `RwLock`。

考虑到我们的多线程图，结果过程流如下所示：

```
    thread
           +  +--------+
      T1   |  |   R1   |
           |  +--------+
      T2   |  |   R2   |
           |  +-----------------+
      T3   |           |   W1   |
           |           +-----------------+
      T4   |                    |   W2   |
           +                    +--------+
              --> read/write reqs over time -->
```

这更好了，因为读者永远不会相互阻塞，但你仍然可以做得更好。

## 第八部分：无锁读取器

在本项目中，你被挑战创建从不加锁的读取器，即使有并发写入者。读请求总是可以被服务，无论写请求如何。（写入者仍可以阻塞其他写入者——除了是一个具有挑战性的并行编程问题外，是否并行写入甚至有意义也是一个难以回答的问题）。

你希望最终得到：

```
    thread
           +  +--------+
      T1   |  |   R1   |
           |  +--------+
      T2   |  |   R2   |
           |  +--------+
      T3   |  |   W1   |
           |  +-----------------+
      T4   |           |   W2   |
           +           +--------+
              --> read/write reqs over time -->
```

如果我们能实现这一点，那么我们的读者将是无锁的：即使单个读者因等待文件系统数据而阻塞，所有其他操作，读取和写入，都可以继续。不幸的是，这仍然不足以保证系统总是能服务读请求。想想如果我们的线程池大小为 `N`，有 `N` 个阻塞的写请求会发生什么。你稍后必须解决这个问题。现在，你专注于从读者中移除锁。

与 `Mutex` 和 `RwLock` 不同，没有单一的包装类型可以应用于整个任意共享状态以实现并发读写（至少，同时保持高性能）。

这意味着我们需要考虑 `SharedKvStore` 的每个字段是如何使用的，并选择正确的同步方案，以允许所有线程尽可能多地取得进展，同时仍保持数据的逻辑一致性。

这才是真正开始多线程的艰难推理的地方。如果你移除了那个大锁，Rust 仍将保护你免受[_数据竞争_]，但它不会帮助你维护数据存储所需的字段之间的逻辑一致性。

[_数据竞争_]: https://blog.regehr.org/archives/490

因此，在考虑解决方案之前，让我们思考一下我们的需求。我们需要：

- 在多个线程上同时从索引和磁盘读取；
- 将命令写入磁盘，同时维护索引；
- 在写入的同时读取，因此
- 一般来说，为了保证读者在并行读取时始终看到一致的状态，这意味着，
  - 维护索引中的日志指针始终指向日志中有效命令的不变量，
  - 维护其他簿记的适当不变量，如以下示例中的 `uncompacted` 变量；
- 定期压缩我们的磁盘数据，同时维护读者的不变量。

本节的其余部分是关于各种主题的背景，这些主题将有助于实现上述目标，但这是本项目剩余部分的全部目标：修改 `KvStore` 以在写入时并发读取。

### 解释我们的示例数据结构

为了具体讨论这一点，我们需要一个我们试图保护的数据示例以及我们试图维护的不变量。因此，这里是一个 `KvStore` 实现及其字段的示例。

```rust
pub struct KvStore {
    /// 日志和其他数据的目录
    path: PathBuf,
    /// 日志读取器
    reader: BufReaderWithPos<File>,
    /// 日志写入器
    writer: BufWriterWithPos<File>,
    /// 从键到日志指针的内存索引
    index: BTreeMap<String, CommandPos>,
    /// 表示在压缩期间可以删除的“陈旧”命令的字节数
    uncompacted: u64,
}
```

这是本项目示例的简化版本。

字段的目的应该相当清楚：

`path: PathBuf` 只是存储日志的目录路径。它从不改变——它是不可变的，而不可变类型在 Rust 中是 `Sync`，因此甚至不需要任何保护。每个线程都可以通过共享引用同时读取它。

`readers: HashMap<u64, BufReaderWithPos<File>>` 是当前日志文件的读取句柄。在压缩后，它需要更改为新的日志文件。

`writer: BufWriterWithPos<File>` 是当前日志文件的写入句柄。因此，任何写入都需要对 `writer` 的可变访问，压缩过程需要更改 `writer` 和 `current_gen`。

`index: BTreeMap<String, CommandPos>` 是数据库中每个键到其在索引文件中位置的内存索引。它被每个读取线程读取，也被每个写入线程写入，可能包括在压缩期间。

`uncompacted: u64` 仅计算日志中已被后续写入命令取代的“陈旧”命令的数量，以知道何时触发压缩。

在以前的项目中，我们不必担心写入、读取和压缩产生不一致结果的交互，因为它们都在同一个线程上发生。现在，如果你不仔细选择数据结构及其使用方式，很容易破坏数据库的状态。

### 拆分锁的策略

高级并行编程的关键是了解可用的工具以及何时使用它们。以下是我们在实现本项目时发现的一些有用技术，其中一些你也将需要。它们在上述示例数据结构的背景下讨论。

#### 理解并维护顺序一致性

（注意，“顺序一致性”有精确的含义，但这里我们只是泛泛地谈论确保需要按特定顺序发生的操作确实如此）。

推理并行程序主要是理解代码中的“先发生”关系。在此线程中，我需要在其他人之前看到哪些共享数据结构的变化？我需要在其他人之前向其他线程暴露哪些共享数据结构的变化？我如何确保这一点？

在单线程代码中，推理任何特定行代码之前发生的事情是微不足道的——如果代码写成在之前发生，那么它就在之前发生，否则就在之后发生。但这实际上并不正确，即使在单线程代码中：CPU 和编译器都会重新排序代码以使其运行更快，CPU 操作机器代码，编译器在其生成机器代码之前的内部表示上操作。实际上，执行的代码顺序与你编写的顺序不同，它只看起来像你写的那样运行，因为 CPU 和编译器跟踪_数据依赖性_，不会重新排序依赖于其他操作的操作。

在多线程代码中，编译器和 CPU 仍会根据与单线程代码相同的假设重新排序代码，除非你通过同步类型和操作告诉编译器不允许重新排序，否则你的代码将完全崩溃。

任何必须在另一个之前或之后发生的操作必须明确安排，无论是通过锁、原子操作或其他方式。

在我们的示例中，很明显，写入文件和写入索引必须以特定顺序被看到——如果索引在文件之前更新会发生什么？同样，我们的示例包含另一个状态位，`uncompacted`。计算不准确的 `uncompacted` 大小有什么影响？如果 `uncompacted` 值在数据提交到文件之前被看到，可能并不那么糟糕，但这是一个必须独立为每个同步值做出的决定。

#### 识别不可变值

你可能已经读了很多关于 Rust 中不可变性的内容，以及不可变值如何轻松在线程之间共享（它们是 `Sync`）。不可变值是并发的最佳选择——只需将它们放在 `Arc` 后面，然后不再考虑它们。

在我们的示例中，`PathBuf` 是不可变的。

#### 复制值而非共享

有时在 Rust 中克隆有坏名声，特别是克隆具有任意大小的类型，如 `String` 和 `Vec`。但克隆通常是完全合理的：在某些情况下避免克隆可能非常困难，而 CPU 在复制内存缓冲区方面非常擅长。此外，考虑到我们的用例，支持服务器所需的状态副本数量由线程池中的线程数量限定。

在我们的示例中，再次 `PathBuf` 是易于克隆的。

不太明显的是，考虑如何在线程之间共享文件访问。[`File`] 类型需要对读取和写入进行可变访问。因此，要在多个线程之间共享它，需要一个授予该可变访问权限的锁。但 `File` 到底是什么？它实际上不是一个文件——它只是一个到磁盘上物理资源的句柄，并且可以同时打开多个到同一文件的句柄。注意 `File` 的 API——它不实现 `Clone`，虽然它确实有这个诱人的 [`try_clone`] 方法，但其语义对多线程应用程序有一些复杂的含义。在 `File` 上的查找是否会影响由 `try_clone` 创建的另一个 `File`？请考虑 `File::open` 和 `try_clone` 创建的 `File` 之间的区别。使用 `try_clone` 或 `File::open`，由你选择。[`pread`] 可能有所帮助。

[`File`]: https://doc.rust-lang.org/std/fs/struct.File.html
[`try_clone`]: https://doc.rust-lang.org/std/fs/struct.File.html#method.try_clone
[`pread`]: https://stackoverflow.com/questions/1687275/what-is-the-difference-between-read-and-pread-in-unix

#### 按角色拆分数据结构

在我们的用例中，我们有两个明确的角色：读者和写者（可能还有一个用于压缩者）。将读者和写者逻辑分离为它们自己的并发类型在 Rust 中很常见。读者有自己的数据集工作，写者也有自己的，这为封装提供了很好的机会，所有读取操作属于一种类型，所有写入操作属于另一种类型。

做出这种区分将进一步明确哪些资源被两者访问，因为读者和写者都将携带这些资源的共享句柄。

#### 使用专用的并发数据结构

知道有哪些工具可用以及在什么情况下使用它们可能是并行编程中最困难的部分。除了学校里教的最基本的锁类型，同步数据类型变得越来越专业化。

在本项目中，由于内存索引是某种关联数据结构（又称“映射”），如树或哈希表，自然会问是否存在并发关联数据结构。

确实存在，使用它们是完成本项目的关键。

但你怎么知道呢？第一步是问是否存在并发映射。你可以在 [Rust Discord] 的 `#beginners` 中这样做，但在此情况下，在网络上搜索“并发映射”肯定会给出答案。

这是容易的部分，找到 Rust 中正确的并发映射则更难。一个好的第一步是学习 [libs.rs]。libs.rs 就像 crates.io，但 crates.io 包含所有发布的库，而 libs.rs 是经过精选的，只包含由……嗯，某人高度评价的库。因此，如果它在 libs.rs 上，那就是一个库可用的指示，另一个是 [crates.io] 上的下载次数——一般来说，下载次数越多的 crate 测试得越多。下载次数可以被视为对 crate 的“背书”人数的粗略代理。最后，询问聊天总是个好主意。

[Rust Discord]: https://doc.rust-lang.org/std/fs/struct.File.html#method.try_clone
[libs.rs]: https://libs.rs
[crates.io]: https://crates.io

#### 推迟清理到稍后

像克隆一样，垃圾收集在 Rust 中经常受到批评——避免 GC 几乎是 Rust 存在的全部原因。但众所周知，实际上，垃圾收集无法避免，“垃圾收集”和“内存回收”几乎是同义词，每种语言都使用混合的垃圾收集策略。在 GC 谱系的一端，在没有自动内存管理的语言中，如 C，垃圾收集完全留给程序员，例如通过 `malloc` 和 `free`。在另一端是垃圾收集语言，如 Java，其中所有内存都由单个通用垃圾收集器收集。

但实际上，C 中的内存管理和回收并非全部通过 `malloc`/`free` 完成，Java 中的内存管理也并非全部通过 GC 完成。仅举一个微不足道的例子，高性能应用程序在两者中都经常依赖专门的[区域]，其中分配可以被重用以及批量释放，以优化其内存访问模式。

[区域]: https://www.quora.com/In-C++-what-is-a-memory-arena

同样在 Rust 中，并非所有内存都是确定性释放的。简单的例子是实现[资源计数]的 [`Rc`] 和 [`Arc`] 类型，这是一种简单的垃圾收集。

[`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
[`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
[引用计数]: https://en.wikipedia.org/wiki/Reference_counting

全局垃圾收集器最大的好处之一是它们使许多无锁数据结构成为可能。学术文献中描述的许多无锁数据结构依赖于 GC 运行。适应无锁算法不依赖 GC 的需求是 [`crossbeam`] 库及其 [`epoch`] 类型的原始动机。

[`crossbeam`]: https://github.com/crossbeam-rs/crossbeam
[`epoch`]: https://docs.rs/crossbeam/0.7.1/crossbeam/epoch/index.html

所有这些都说明，垃圾收集有多种形式，其延迟资源清理到未来时间的基本策略在许多场景中是强大的。

当你无法弄清楚如何立即执行某些并发工作时，问“我能否稍后再做这个？”可能是有用的。

#### 使用原子操作共享标志和计数器

在底层，大多数并发数据结构是使用[原子操作]或“原子”实现的。原子操作作用于单个内存单元，通常在 8 到 128 字节之间，常见的是字大小（与指针相同数量的字节，以及 Rust 的 `usize` 类型）。如果两个线程正确使用原子操作，则一个线程的写入结果会立即对另一个线程的读取可见。除了使读取或写入立即可见外，原子操作还约束了编译器和 CPU 可能如何重新排序指令，在 Rust 中通过 [`Ordering`] 标志实现。

[原子操作]: https://preshing.com/20130618/atomic-vs-non-atomic-operations/
[`atomic`]: https://doc.rust-lang.org/std/sync/atomic/
[`Ordering`]: https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html

当从锁的粗粒度并行性转向更细粒度的并行性时，通常需要用原子操作增强现成的并发数据结构。

### 实现无锁读取器

这有很多背景。希望那里有很多内容可以思考并引导你走向正确的方向。现在轮到你了：

_修改 `KvStore` 以在写入时并发读取_。

之后……

干得好，朋友。享受一个美好的休息吧。


<!--

### 一些共享数据而不使用大锁的想法


- TODO: https://gitlab.redox-os.org/redox-os/chashmap
- TODO: https://github.com/jonhoo/rust-evmap
- https://github.com/4lDO2/evc
- crossbeam-skiplist
- atomics
- invariants
- concurrent maps https://gitlab.nebulanet.cc/xacrimon/rs-hm-bench

这里的一些数据类型有等效的并发类型：
例如，一个 `u64` 可以替换为一个

_好吧，我希望你已经准备好了。去尽可能多地移除这个类型的锁和竞争_

这里没有新的测试用例需要完成，但一些早期的测试用例将以具有挑战性的方式测试这个新数据结构，你之前编写的基准测试将对这个实现施加巨大压力。


## 第九部分：基准测试无锁数据结构

TODO：在前面的部分中做一个读写基准测试，
      验证键的总和
TODO：确保基准测试部分始终提到要断言结果
-->


<!--
---


## 扩展 1：后台压缩

- 讨论文件和并发的问题
- 将压缩移到后台线程
- 需要重构以前的项目以使用多个日志
- 这应该相当具有挑战性
-->


<!--

## 背景阅读建议

- 调度策略
- 共享可变状态，特别是在多线程上下文中
- 线程池
- 某些关于并行性和并发性的内容
- 某些解释 Arc<Mutex> 的内容
- 某些关于内部和外部可变性区别的内容，如果有包含并发性则更好
- 并发映射比较 https://gitlab.nebulanet.cc/xacrimon/rs-hm-bench

## TODOs

- 一个并发映射或跳表会比一个互斥哈希表更好，但似乎没有一个生产质量的 crate
- 是否有某种新的测量方式可以对线程池进行，除了 criterion 基准测试？
- 线程池中线程的恐慌处理

- 在 `KvStore(Arc<SharedKvStore>)` 示例中讨论访问类型的模式，特别是不要被诱惑使用 `Deref`。
- 在某处提及 condvars

-->