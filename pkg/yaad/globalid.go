package yaad

var globalIDCtr = 0

func NextID() int {
	globalIDCtr++
	return globalIDCtr
}
