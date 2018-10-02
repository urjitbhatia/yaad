package yaad

import "time"

// An Item is something we manage in a priority queue.
type Item struct {
	value    interface{} // The value of the item; Spoke or Job.
	priority time.Time   // The priority of the item in the queue.
	// The index is needed by update and is maintained by the heap.Interface methods.
	index int // The index of the item in the heap.
}

// A PriorityQueue implements heap.Interface and holds Items.
type PriorityQueue []*Item

func (pq PriorityQueue) Len() int { return len(pq) }

// Less defines item ordering. Priority is defined by trigger time in the future
func (pq PriorityQueue) Less(i, j int) bool {
	// We want Pop to give us the item nearest in time, not highest.
	// if i starts AFTER j, i has lower priority
	return pq[i].priority.After(pq[j].priority)
}

func (pq PriorityQueue) Swap(i, j int) {
	pq[i], pq[j] = pq[j], pq[i]
	pq[i].index = i
	pq[j].index = j
}

// Push an item to this PriorityQueue
func (pq *PriorityQueue) Push(x interface{}) {
	n := len(*pq)
	item := x.(*Item)
	item.index = n
	*pq = append(*pq, item)
}

// Pop the item with the closest trigger time (priority)
func (pq *PriorityQueue) Pop() interface{} {
	old := *pq
	n := len(old)
	item := old[n-1]
	item.index = -1 // for safety
	*pq = old[0 : n-1]
	return item
}
