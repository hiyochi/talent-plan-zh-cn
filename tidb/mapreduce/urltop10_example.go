package main

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
)

// ExampleURLTop10 生成 RoundsArgs，用于获取出现频率最高的 10 个 URL。
// 此方法包含两个阶段。
// 第一阶段进行 URL 计数。
// 第二阶段对第一阶段生成的结果进行排序，并获取出现频率最高的 10 个 URL。
func ExampleURLTop10(nWorkers int) RoundsArgs {
	var args RoundsArgs
	// 第一阶段：进行 URL 计数
	args = append(args, RoundArgs{
		MapFunc:    ExampleURLCountMap,
		ReduceFunc: ExampleURLCountReduce,
		NReduce:    nWorkers,
	})
	// 第二阶段：排序并获取出现频率最高的 10 个 URL
	args = append(args, RoundArgs{
		MapFunc:    ExampleURLTop10Map,
		ReduceFunc: ExampleURLTop10Reduce,
		NReduce:    1,
	})
	return args
}

// ExampleURLCountMap 是第一阶段的 map 函数
func ExampleURLCountMap(filename string, contents string) []KeyValue {
	lines := strings.Split(contents, "\n")
	kvs := make([]KeyValue, 0, len(lines))
	for _, l := range lines {
		l = strings.TrimSpace(l)
		if len(l) == 0 {
			continue
		}
		kvs = append(kvs, KeyValue{Key: l})
	}
	return kvs
}

// ExampleURLCountReduce 是第一阶段的 reduce 函数
func ExampleURLCountReduce(key string, values []string) string {
	return fmt.Sprintf("%s %s\n", key, strconv.Itoa(len(values)))
}

// ExampleURLTop10Map 是第二阶段的 map 函数
func ExampleURLTop10Map(filename string, contents string) []KeyValue {
	lines := strings.Split(contents, "\n")
	kvs := make([]KeyValue, 0, len(lines))
	for _, l := range lines {
		kvs = append(kvs, KeyValue{"", l})
	}
	return kvs
}

// ExampleURLTop10Reduce 是第二阶段的 reduce 函数
func ExampleURLTop10Reduce(key string, values []string) string {
	cnts := make(map[string]int, len(values))
	for _, v := range values {
		v := strings.TrimSpace(v)
		if len(v) == 0 {
			continue
		}
		tmp := strings.Split(v, " ")
		n, err := strconv.Atoi(tmp[1])
		if err != nil {
			panic(err)
		}
		cnts[tmp[0]] = n
	}

	us, cs := TopN(cnts, 10)
	buf := new(bytes.Buffer)
	for i := range us {
		fmt.Fprintf(buf, "%s: %d\n", us[i], cs[i])
	}
	return buf.String()
}