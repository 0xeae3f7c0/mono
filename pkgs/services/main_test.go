package main

import (
	"testing"

	"github.com/moby/moby/api/types/container"
	"github.com/stretchr/testify/assert"
)

func TestByImageCountsAndSorts(t *testing.T) {
	got := byImage([]container.Summary{{Image: "redis"}, {Image: "nginx"}, {Image: "redis"}, {Image: "app"}})
	assert.Equal(t, []ImageCount{{"redis", 2}, {"app", 1}, {"nginx", 1}}, got)
}
