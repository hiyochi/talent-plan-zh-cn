package main

import (
	"math/rand"
	"sort"
	"testing"
	"time"

	"github.com/pingcap/check"
)

var _ = check.Suite(&sortTestSuite{})

// 测试入口函数，用于运行 check 测试框架
func TestT(t *testing.T) {
	check.TestingT(t)
}

// 准备测试数据：使用当前时间作为随机种子，填充切片为随机的 int64 值
func prepare(src []int64) {
	rand.Seed(time.Now().Unix())
	for i := range src {
		src[i] = rand.Int63()
	}
}

type sortTestSuite struct{}

// 测试归并排序算法的正确性
func (s *sortTestSuite) TestMergeSort(c *check.C) {
	lens := []int{1, 3, 5, 7, 11, 13, 17, 19, 23, 29, 1024, 1 << 13, 1 << 17, 1 << 19, 1 << 20}

	for i := range lens {
		src := make([]int64, lens[i])
		expect := make([]int64, lens[i])
		prepare(src)
		copy(expect, src)
		MergeSort(src)
		sort.Slice(expect, func(i, j int) bool { return expect[i] < expect[j] })
		for i := 0; i < len(src); i++ {
			c.Assert(src[i], check.Equals, expect[i])
		}
	}
}