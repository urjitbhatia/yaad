package yaad

import (
	"container/heap"
	"errors"
	"sync"
	"time"

	"github.com/sirupsen/logrus"
)

const (
	hundredYears = time.Hour * 24 * 365 * 100
)

// Hub is a time ordered collection of spokes
type Hub struct {
	spokeSpan time.Duration
	spokeMap  map[spokeBound]*Spoke // quick lookup map
	spokes    *PriorityQueue

	pastSpoke    *Spoke // Permanently pinned to the past
	currentSpoke *Spoke // The current spoke

	reservedJobs map[string]*Job // This could also be a spoke that order by TTL - optimize later

	removedJobsCount uint64
	lock             *sync.Mutex
}

// NewHub creates a new hub where adjacent spokes lie at the given
// spokeSpan duration boundary.
func NewHub(spokeSpan time.Duration) *Hub {
	h := &Hub{
		spokeSpan:        spokeSpan,
		spokeMap:         make(map[spokeBound]*Spoke),
		spokes:           &PriorityQueue{},
		pastSpoke:        NewSpoke(time.Now().Add(-1*hundredYears), time.Now().Add(hundredYears)),
		currentSpoke:     nil,
		reservedJobs:     make(map[string]*Job),
		removedJobsCount: 0,
		lock:             &sync.Mutex{},
	}
	heap.Init(h.spokes)

	logrus.WithFields(logrus.Fields{
		"start": h.pastSpoke.start.String(),
		"end":   h.pastSpoke.end.String(),
	}).Debug("Created hub with past spoke")

	go h.StatusPrinter()

	return h
}

// PendingJobsCount return the number of jobs currently pending
func (h *Hub) PendingJobsCount() int {
	count := h.pastSpoke.PendingJobsLen()
	for _, v := range h.spokeMap {
		count += v.PendingJobsLen()
	}

	return count
}

// CancelJob cancels a job if found. Calls are noop for unknown jobs
func (h *Hub) CancelJob(jobID string) error {
	h.lock.Lock()
	defer h.lock.Unlock()

	logrus.Debug("cancel: ", jobID)
	// Search if this job is reserved
	if _, ok := h.reservedJobs[jobID]; ok {
		logrus.Debug("found in reserved: ", jobID)
		delete(h.reservedJobs, jobID)
		h.removedJobsCount++
		return nil
	}

	s, err := h.FindOwnerSpoke(jobID)
	if err != nil {
		logrus.Debug("cancel found no owner spoke: ", jobID)
		return err
	}
	logrus.Debug("cancel found owner spoke: ", jobID)
	s.CancelJob(jobID)
	h.removedJobsCount++
	return nil
}

// FindOwnerSpoke returns the spoke that owns this job
func (h *Hub) FindOwnerSpoke(jobID string) (*Spoke, error) {

	if h.pastSpoke.OwnsJob(jobID) {
		return h.pastSpoke, nil
	}

	// Checking the current spoke - lock the hub
	if h.currentSpoke != nil && h.currentSpoke.OwnsJob(jobID) {
		return h.currentSpoke, nil
	}

	for _, v := range h.spokeMap {
		if v.OwnsJob(jobID) {
			return v, nil
		}
	}
	return nil, errors.New("Cannot find job owner spoke")
}

// addSpoke adds spoke s to this hub
func (h *Hub) addSpoke(s *Spoke) {
	h.spokeMap[s.spokeBound] = s
	heap.Push(h.spokes, s.AsPriorityItem())
}

// Next returns the next job that is ready now or returns nil.
func (h *Hub) Next() *Job {
	h.lock.Lock()
	defer h.lock.Unlock()

	pastLocker := h.pastSpoke.GetLocker()
	pastLocker.Lock()
	defer pastLocker.Unlock()

	// Find a job in past spoke
	j := h.pastSpoke.Next()
	if j != nil {
		h.reserve(j)
		logrus.Debug("Got job from past spoke")
		return j
	}
	// Checked past spoke

	// Fix the heap
	heap.Init(h.spokes)

	// Find a job in current spoke
	// If current is empty and now expired, prune it...
	if h.currentSpoke != nil {
		if h.currentSpoke.PendingJobsLen() == 0 && h.currentSpoke.AsTemporalState() == Past {
			logrus.Debug("pruning the current spoke")
			// This routine could be unfortunate - it found a currentspoke that was expired
			// so it has the pay the price finding the next candidate
			delete(h.spokeMap, h.currentSpoke.spokeBound)
			h.currentSpoke = nil
		}
	}

	// No currently assigned spoke
	if h.currentSpoke == nil {
		if h.spokes.Len() == 0 {
			logrus.Debug("No spokes")
			// No spokes - can't do anything. Return
			return nil
		}

		// New current candidate
		item := h.spokes.AtIdx(0)
		current := item.value.(*Spoke)
		switch current.AsTemporalState() {
		case Future:
			// Next in time is still not current. Can't do anything. Return
			return nil
		case Past, Current:
			// We have found a new current spoke
			h.currentSpoke = current
			// Pop it from the queue - this is now a current spoke
			heap.Pop(h.spokes)
		}
	}

	// Read from current spoke

	// Assert - At this point, hub should have a current spoke
	if h.currentSpoke == nil {
		logrus.Panic("Unreachable state :: hub has a nil spoke after candidate search")
	}

	currentLocker := h.currentSpoke.GetLocker()
	currentLocker.Lock()
	defer currentLocker.Unlock()

	j = h.currentSpoke.Next()
	if j == nil {
		// no job - return
		logrus.Debug("No job in current spoke")
		return nil
	}

	logrus.Debug("reserving job: ", j.id)
	h.reserve(j)

	return j
}

func (h *Hub) reserve(j *Job) {
	h.reservedJobs[j.id] = j
}

func (h *Hub) mergeQueues(pq *PriorityQueue) {
	for pq.Len() > 0 {
		i := heap.Pop(pq)
		h.spokes.Push(i)
	}
}

// Prune clears spokes which are expired and have no jobs
// returns the number of spokes pruned
func (h *Hub) Prune() int {
	pruned := 0
	for k, v := range h.spokeMap {
		if v.IsExpired() && v.PendingJobsLen() == 0 {
			delete(h.spokeMap, k)
		}
		pruned++
	}

	return pruned
}

// AddJob to this hub. Hub should never reject a job - this method will panic if that happens
func (h *Hub) AddJob(j *Job) error {

	switch j.AsTemporalState() {
	case Past:
		pastLocker := h.pastSpoke.GetLocker()
		pastLocker.Lock()
		defer pastLocker.Unlock()

		logrus.WithField("JobID", j.ID).Debug("Adding job to past spoke")
		err := h.pastSpoke.AddJob(j)
		if err != nil {
			logrus.WithError(err).Error("Past spoke rejected job. This should never happen")
			return err
		}
	case Future:
		// Lock hub so that current spoke isn't replaced
		h.lock.Lock()
		defer h.lock.Unlock()

		// Lock current spoke so that add fixes the PQ as it adds
		if h.currentSpoke != nil {
			currLocker := h.currentSpoke.GetLocker()
			currLocker.Lock()
			defer currLocker.Unlock()

			if h.currentSpoke.ContainsJob(j) {
				err := h.currentSpoke.AddJob(j)
				if err != nil {
					logrus.WithError(err).Error("Current spoke rejected job. This should never happen")
					return err
				}
				return nil
			}
		}

		// Search for a spoke that can take ownership of this job
		// Reads are still going to be ordered anyways
		jobBound := j.AsBound(h.spokeSpan)
		candidate, ok := h.spokeMap[jobBound]
		if ok {
			// Found a candidate that can take this job
			err := candidate.AddJob(j)
			if err != nil {
				logrus.WithError(err).Error("Hub should always accept a job. No spoke accepted")
				return err
			}
			// Accepted, all done...
			return nil
		}

		// Time to create a new spoke for this job
		s := NewSpoke(jobBound.start, jobBound.end)
		err := s.AddJob(j)
		if err != nil {
			logrus.WithError(err).Error("Hub should always accept a job. No spoke accepted")
			return err
		}

		// h is still locked here so it's ok
		h.addSpoke(s)
	}
	return nil
}

// Status prints the state of the spokes of this hub
func (h *Hub) Status() {
	logrus.Info("-------------------------------------------------------------")
	logrus.Infof("Hub has %d spokes", len(h.spokeMap))
	logrus.Infof("Hub has %d total jobs", h.PendingJobsCount())
	h.lock.Lock()
	logrus.Infof("Hub has %d reserved jobs", len(h.reservedJobs))
	h.lock.Unlock()
	logrus.Infof("Hub has %d removed jobs", h.removedJobsCount)
	logrus.Infof("Past spoke has %d jobs", h.pastSpoke.PendingJobsLen())
	for _, s := range h.spokeMap {
		logrus.Infof("Spoke %s has %d jobs", s.id, s.PendingJobsLen())
		logrus.Debugf("Spoke %s start: %s end %s", s.id, s.start.String(), s.end.String())
	}
	logrus.Info("-------------------------------------------------------------")
}

// StatusPrinter starts a status printer that prints hub stats over some time interval
func (h *Hub) StatusPrinter() {
	t := time.NewTicker(time.Second * 10)
	for range t.C {
		h.Status()
	}
}
