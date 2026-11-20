package main

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes/fake"
)

func pod(ns, name string, phase corev1.PodPhase) *corev1.Pod {
	return &corev1.Pod{ObjectMeta: metav1.ObjectMeta{Namespace: ns, Name: name}, Status: corev1.PodStatus{Phase: phase}}
}

func TestReportGroupsAndSorts(t *testing.T) {
	client := fake.NewClientset(
		pod("web", "a", corev1.PodRunning),
		pod("web", "b", corev1.PodRunning),
		pod("jobs", "c", corev1.PodSucceeded),
		pod("web", "d", corev1.PodPending),
	)
	counts, err := report(context.Background(), client)
	require.NoError(t, err)
	assert.Equal(t, []Count{
		{"jobs", "Succeeded", 1},
		{"web", "Pending", 1},
		{"web", "Running", 2},
	}, counts)
}
