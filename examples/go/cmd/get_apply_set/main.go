package main

import (
	"errors"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"
	"sync"

	"github.com/bradfitz/gomemcache/memcache"
	getopt "github.com/pborman/getopt/v2"
)

type fnSig func(*memcache.Client, string, nextSig)

func nonatomic(conn *memcache.Client, key string, nextStr nextSig) {
	res, err := conn.Get(key)
	if err != nil {
		if errors.Is(err, memcache.ErrCacheMiss) {
			val := nextStr(nil)
			conn.Set(&memcache.Item{Key: key, Value: val})
		}
	} else {
		val := nextStr(&res.Value)
		conn.Set(&memcache.Item{Key: key, Value: val})
	}
}

func atomic(conn *memcache.Client, key string, nextStr nextSig) {
	for {
		res, err := conn.Get(key)
		if err != nil {
			if errors.Is(err, memcache.ErrCacheMiss) {
				val := nextStr(nil)
				err = conn.Add(&memcache.Item{Key: key, Value: val})
				if err == nil {
					return
				}
			}
		} else {
			res.Value = nextStr(&res.Value)
			err = conn.CompareAndSwap(res)
			if err == nil {
				return
			}
		}
	}
}

func repeat(wg *sync.WaitGroup, conn *memcache.Client, nextval nextSig, key string, iter int, fn fnSig) {
	defer wg.Done()

	for i := 0; i < iter; i++ {
		fn(conn, key, nextval)
	}
}

func nextInt(v int) int {
	if v%2 == 0 {
		return v + 3
	} else {
		return v + 1
	}
}

type nextSig func(*[]byte) []byte

func nextStr(s *[]byte) []byte {
	res := "0"

	if s != nil {
		val, err := strconv.Atoi(string(*s))
		if err != nil {
			os.Exit(2)
		}

		res = strconv.Itoa(nextInt(val))
	}

	return []byte(res)
}

var m = map[string]fnSig{
	"atomic":    atomic,
	"nonatomic": nonatomic,
}

func joinKeys(d map[string]fnSig) string {
	keys := make([]string, 0, len(d))
	for k := range d {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return strings.Join(keys, "|")
}

var (
	host   string
	port   string
	iter   int
	key    string
	method string
)

func init() {
	getopt.FlagLong(&host, "host", 0, "memcached host", "host").Mandatory()
	getopt.FlagLong(&port, "port", 0, "memcached port", "port").Mandatory()
	getopt.FlagLong(&iter, "iter", 0, "iterations", "iterations").Mandatory()
	getopt.FlagLong(&key, "key", 0, "memcached key", "key").Mandatory()
	getopt.FlagLong(&method, "method", 0, "method to call", joinKeys(m)).Mandatory()
}

func main() {
	getopt.Parse()

	if _, exists := m[method]; !exists {
		getopt.PrintUsage(os.Stderr)
		os.Exit(1)
	}

	conn := memcache.New(fmt.Sprintf("%s:%s", host, port))
	concurrency := 2

	conn.Delete(key)

	var wg sync.WaitGroup
	for i := 1; i <= concurrency; i++ {
		wg.Add(1)
		go repeat(&wg, conn, nextStr, key, iter, m[method])
	}
	wg.Wait()

	res, err := conn.Get(key)
	if err != nil {
		os.Exit(1)
	}
	actual := string(res.Value)

	var expected *[]byte
	for i := 0; i < concurrency*iter; i++ {
		val := nextStr(expected)
		expected = &val
	}

	fmt.Printf("expected: %s, actual: %s\n", *expected, actual)
}
