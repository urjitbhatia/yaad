package yaad_test

import (
	"container/heap"
	"time"

	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"

	uuid "github.com/satori/go.uuid"
	. "github.com/urjitbhatia/yaad/pkg/yaad"
)

var _ = Describe("Test jobs", func() {
	Context("Basic job tests", func() {
		It("can create a job", func() {
			b := []byte("foo")
			j := NewJob(uuid.NewV4().String(), time.Now(), &b)
			Expect(j.IsReady()).To(BeTrue())
		})

		It("can create spoke bounds from job trigger time", func() {
			t := time.Unix(0, 0)
			j := NewJobAutoID(t.Add(time.Second*15), nil)

			sb := j.AsBound(time.Second)

			Expect(sb.Start().Before(j.TriggerAt()))
			Expect(sb.End().After(j.TriggerAt()))
		})
	})

	Context("Job Ordering", func() {
		It("orders jobs correctly", func() {
			t := time.Now()
			jone := NewJobAutoID(t.Add(1), nil)
			jtwo := NewJobAutoID(t.Add(20), nil)
			jthree := NewJobAutoID(t.Add(50), nil)
			ordList := []*Job{jone, jtwo, jthree}

			jobs := &PriorityQueue{jtwo.AsPriorityItem(), jone.AsPriorityItem(), jthree.AsPriorityItem()}
			heap.Init(jobs)

			for _, job := range ordList {
				j := heap.Pop(jobs).(*Item).Value().(*Job)
				Expect(j.ID()).To(Equal(job.ID()))
			}
		})
	})
})