package main

import (
	"bufio"
	"fmt"
	"io/ioutil"
	"os"
	"path"
	"sort"
	"strings"
)

// RoundArgs 包含在一次 map-reduce 轮次中使用的参数。
type RoundArgs struct {
	MapFunc    MapF
	ReduceFunc ReduceF
	NReduce    int
}

// RoundsArgs 表示在多轮 map-reduce 中使用的参数。
type RoundsArgs []RoundArgs

type urlCount struct {
	url string
	cnt int
}

// TopN 返回 urlCntMap 中前 N 个 URL。
func TopN(urlCntMap map[string]int, n int) ([]string, []int) {
	ucs := make([]*urlCount, 0, len(urlCntMap))
	for k, v := range urlCntMap {
		ucs = append(ucs, &urlCount{k, v})
	}
	sort.Slice(ucs, func(i, j int) bool {
		if ucs[i].cnt == ucs[j].cnt {
			return ucs[i].url < ucs[j].url
		}
		return ucs[i].cnt > ucs[j].cnt
	})
	urls := make([]string, 0, n)
	cnts := make([]int, 0, n)
	for i, u := range ucs {
		if i == n {
			break
		}
		urls = append(urls, u.url)
		cnts = append(cnts, u.cnt)
	}
	return urls, cnts
}

// CheckFile 检查这两个文件是否相同。
func CheckFile(expected, got string) (string, bool) {
	c1, err := ioutil.ReadFile(expected)
	if err != nil {
		panic(err)
	}
	c2, err := ioutil.ReadFile(got)
	if err != nil {
		panic(err)
	}
	s1 := strings.TrimSpace(string(c1))
	s2 := strings.TrimSpace(string(c2))
	if s1 == s2 {
		return "", true
	}

	errMsg := fmt.Sprintf("expected:\n%s\n, but got:\n%s\n", c1, c2)
	return errMsg, false
}

// CreateFileAndBuf 打开或创建一个指定文件用于写入。
func CreateFileAndBuf(fpath string) (*os.File, *bufio.Writer) {
	dir := path.Dir(fpath)
	os.MkdirAll(dir, 0777)
	f, err := os.OpenFile(fpath, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0666)
	if err != nil {
		panic(err)
	}
	return f, bufio.NewWriterSize(f, 1<<20)
}

// OpenFileAndBuf 打开一个指定文件用于读取。
func OpenFileAndBuf(fpath string) (*os.File, *bufio.Reader) {
	f, err := os.OpenFile(fpath, os.O_RDONLY, 0666)
	if err != nil {
		panic(err)
	}
	return f, bufio.NewReader(f)
}

// WriteToBuf 将字符串写入缓冲区。
func WriteToBuf(buf *bufio.Writer, strs ...string) {
	for _, str := range strs {
		if _, err := buf.WriteString(str); err != nil {
			panic(err)
		}
	}
}

// SafeClose 刷新缓冲区并关闭文件。
func SafeClose(f *os.File, buf *bufio.Writer) {
	if buf != nil {
		if err := buf.Flush(); err != nil {
			panic(err)
		}
	}
	if err := f.Close(); err != nil {
		panic(err)
	}
}

// FileOrDirExist 简单地测试该文件或目录是否存在。
func FileOrDirExist(p string) bool {
	_, err := os.Stat(p)
	return err == nil
}