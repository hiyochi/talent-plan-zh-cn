package main

import (
	"fmt"
	"log"
	"os"
	"path"
	"runtime"
	"testing"
	"time"
)

func testDataScale() ([]DataSize, []int) {
	// 生成数据规模和映射文件数量的测试组合
	dataSize := []DataSize{1 * MB, 10 * MB, 100 * MB, 500 * MB, 1 * GB}
	nMapFiles := []int{5, 10, 20, 40, 60}
	return dataSize, nMapFiles
}

const (
	dataDir = "/tmp/mr_homework" // 数据文件存储目录
)

func dataPrefix(i int, ds DataSize, nMap int) string {
	// 生成数据文件的前缀路径，格式为：case{索引}-{数据大小}-{映射文件数}
	return path.Join(dataDir, fmt.Sprintf("case%d-%s-%d", i, ds, nMap))
}

func TestGenData(t *testing.T) {
	// 获取所有数据生成器
	gens := AllCaseGenFs()
	dataSize, nMapFiles := testDataScale()
	for k := range dataSize {
		for i, gen := range gens {
			fmt.Printf("generate data file for cast%d, dataSize=%v, nMap=%v\n", i, dataSize[k], nMapFiles[k])
			prefix := dataPrefix(i, dataSize[k], nMapFiles[k])
			gen(prefix, int(dataSize[k]), nMapFiles[k])
		}
	}
}

func TestCleanData(t *testing.T) {
	// 清理所有测试生成的数据文件
	if err := os.RemoveAll(dataDir); err != nil {
		log.Fatal(err)
	}
}

func TestExampleURLTop(t *testing.T) {
	// 使用示例的 URLTop10 实现运行测试
	rounds := ExampleURLTop10(GetMRCluster().NWorkers())
	testURLTop(t, rounds)
}

func TestURLTop(t *testing.T) {
	// 使用自定义的 URLTop10 实现运行测试
	rounds := URLTop10(GetMRCluster().NWorkers())
	testURLTop(t, rounds)
}

func testURLTop(t *testing.T, rounds RoundsArgs) {
	// 检查是否提供了有效的测试轮次参数
	if len(rounds) == 0 {
		t.Fatalf("no rounds arguments, please finish your code")
	}
	mr := GetMRCluster()

	// 运行所有测试用例
	gens := AllCaseGenFs()
	dataSize, nMapFiles := testDataScale()
	for k := range dataSize {
		for i, gen := range gens {
			// 生成测试数据
			prefix := dataPrefix(i, dataSize[k], nMapFiles[k])
			c := gen(prefix, int(dataSize[k]), nMapFiles[k])

			runtime.GC()

			// 执行 MapReduce 轮次
			begin := time.Now()
			inputFiles := c.MapFiles
			for idx, r := range rounds {
				jobName := fmt.Sprintf("Case%d-Round%d", i, idx)
				ch := mr.Submit(jobName, prefix, r.MapFunc, r.ReduceFunc, inputFiles, r.NReduce)
				inputFiles = <-ch
			}
			cost := time.Since(begin)

			// 检查结果文件数量是否正确
			if len(inputFiles) != 1 {
				panic("the length of result file list should be 1")
			}
			result := inputFiles[0]

			// 验证结果文件内容是否符合预期
			if errMsg, ok := CheckFile(c.ResultFile, result); !ok {
				t.Fatalf("Case%d FAIL, dataSize=%v, nMapFiles=%v, cost=%v\n%v\n", i, dataSize[k], nMapFiles[k], cost, errMsg)
			} else {
				fmt.Printf("Case%d PASS, dataSize=%v, nMapFiles=%v, cost=%v\n", i, dataSize[k], nMapFiles[k], cost)
			}
		}
	}
}