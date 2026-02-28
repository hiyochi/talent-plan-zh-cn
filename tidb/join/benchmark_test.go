package main

import "testing"

// BenchmarkJoin 测试 Join 函数的性能
func BenchmarkJoin(b *testing.B) {
	for i := 0; i < b.N; i++ {
		Join("./t/r0.tbl", "./t/r2.tbl", []int{0}, []int{1})
	}
}

// BenchmarkJoinExample 测试 JoinExample 函数的性能
func BenchmarkJoinExample(b *testing.B) {
	for i := 0; i < b.N; i++ {
		JoinExample("./t/r0.tbl", "./t/r2.tbl", []int{0}, []int{1})
	}
}