```go
package main

import (
	"bufio"
	"encoding/json"
	"hash/fnv"
	"io/ioutil"
	"log"
	"os"
	"path"
	"runtime"
	"strconv"
	"sync"
)

// KeyValue 是用于保存传递给 map 和 reduce 函数的键值对的类型。
type KeyValue struct {
	Key   string
	Value string
}

// ReduceF 是 MIT 6.824 LAB1 中的 reduce 函数
type ReduceF func(key string, values []string) string

// MapF 是 MIT 6.824 LAB1 中的 map 函数
type MapF func(filename string, contents string) []KeyValue

// jobPhase 表示任务被调度为 map 任务还是 reduce 任务。
type jobPhase string

const (
	mapPhase    jobPhase = "mapPhase"
	reducePhase          = "reducePhase"
)

type task struct {
	dataDir    string
	jobName    string
	mapFile    string   // 仅用于 map 任务，输入文件
	phase      jobPhase // 当前处于 mapPhase 还是 reducePhase？
	taskNumber int      // 当前阶段中该任务的索引
	nMap       int      // map 任务的数量
	nReduce    int      // reduce 任务的数量
	mapF       MapF     // 本任务中使用的 map 函数
	reduceF    ReduceF  // 本任务中使用的 reduce 函数
	wg         sync.WaitGroup
}

// MRCluster 表示一个 map-reduce 集群。
type MRCluster struct {
	nWorkers int
	wg       sync.WaitGroup
	taskCh   chan *task
	exit     chan struct{}
}

var singleton = &MRCluster{
	nWorkers: runtime.NumCPU(),
	taskCh:   make(chan *task),
	exit:     make(chan struct{}),
}

func init() {
	singleton.Start()
}

// GetMRCluster 返回对 MRCluster 的引用。
func GetMRCluster() *MRCluster {
	return singleton
}

// NWorkers 返回此集群中工作线程的数量。
func (c *MRCluster) NWorkers() int { return c.nWorkers }

// Start 启动此集群。
func (c *MRCluster) Start() {
	for i := 0; i < c.nWorkers; i++ {
		c.wg.Add(1)
		go c.worker()
	}
}

func (c *MRCluster) worker() {
	defer c.wg.Done()
	for {
		select {
		case t := <-c.taskCh:
			if t.phase == mapPhase {
				content, err := ioutil.ReadFile(t.mapFile)
				if err != nil {
					panic(err)
				}

				fs := make([]*os.File, t.nReduce)
				bs := make([]*bufio.Writer, t.nReduce)
				for i := range fs {
					rpath := reduceName(t.dataDir, t.jobName, t.taskNumber, i)
					fs[i], bs[i] = CreateFileAndBuf(rpath)
				}
				results := t.mapF(t.mapFile, string(content))
				for _, kv := range results {
					enc := json.NewEncoder(bs[ihash(kv.Key)%t.nReduce])
					if err := enc.Encode(&kv); err != nil {
						log.Fatalln(err)
					}
				}
				for i := range fs {
					SafeClose(fs[i], bs[i])
				}
			} else {
				// YOUR CODE HERE :)
				// 提示：不要对 ReduceF 返回的结果进行编码，而是直接将结果输出到目标文件，
				// 以便用户能够以他们期望的格式获取结果。
				panic("YOUR CODE HERE")
			}
			t.wg.Done()
		case <-c.exit:
			return
		}
	}
}

// Shutdown 关闭此集群。
func (c *MRCluster) Shutdown() {
	close(c.exit)
	c.wg.Wait()
}

// Submit 将一个任务提交到此集群。
func (c *MRCluster) Submit(jobName, dataDir string, mapF MapF, reduceF ReduceF, mapFiles []string, nReduce int) <-chan []string {
	notify := make(chan []string)
	go c.run(jobName, dataDir, mapF, reduceF, mapFiles, nReduce, notify)
	return notify
}

func (c *MRCluster) run(jobName, dataDir string, mapF MapF, reduceF ReduceF, mapFiles []string, nReduce int, notify chan<- []string) {
	// map 阶段
	nMap := len(mapFiles)
	tasks := make([]*task, 0, nMap)
	for i := 0; i < nMap; i++ {
		t := &task{
			dataDir:    dataDir,
			jobName:    jobName,
			mapFile:    mapFiles[i],
			phase:      mapPhase,
			taskNumber: i,
			nReduce:    nReduce,
			nMap:       nMap,
			mapF:       mapF,
		}
		t.wg.Add(1)
		tasks = append(tasks, t)
		go func() { c.taskCh <- t }()
	}
	for _, t := range tasks {
		t.wg.Wait()
	}

	// reduce 阶段
	// YOUR CODE HERE :D
	panic("YOUR CODE HERE")
}

func ihash(s string) int {
	h := fnv.New32a()
	h.Write([]byte(s))
	return int(h.Sum32() & 0x7fffffff)
}

func reduceName(dataDir, jobName string, mapTask int, reduceTask int) string {
	return path.Join(dataDir, "mrtmp."+jobName+"-"+strconv.Itoa(mapTask)+"-"+strconv.Itoa(reduceTask))
}

func mergeName(dataDir, jobName string, reduceTask int) string {
	return path.Join(dataDir, "mrtmp."+jobName+"-res-"+strconv.Itoa(reduceTask))
}
```