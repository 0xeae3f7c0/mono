// services: Running platform containers on a Docker host (commit platform, pkgs/services)
// Command services shows which platform services are running on a Docker host
// (single-node deployments): running containers grouped by image.
package main

import (
	"context"
	"fmt"
	"os"
	"sort"

	"github.com/moby/moby/api/types/container"
	"github.com/moby/moby/client"
)

// ImageCount is how many running containers use an image.
type ImageCount struct {
	Image      string
	Containers int
}

func byImage(containers []container.Summary) []ImageCount {
	counts := map[string]int{}
	for _, c := range containers {
		counts[c.Image]++
	}
	out := make([]ImageCount, 0, len(counts))
	for image, n := range counts {
		out = append(out, ImageCount{image, n})
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Containers != out[j].Containers {
			return out[i].Containers > out[j].Containers
		}
		return out[i].Image < out[j].Image
	})
	return out
}

func main() {
	cli, err := client.New(client.FromEnv)
	if err != nil {
		fmt.Fprintln(os.Stderr, "docker client:", err)
		os.Exit(1)
	}
	defer cli.Close()
	result, err := cli.ContainerList(context.Background(), client.ContainerListOptions{})
	if err != nil {
		fmt.Fprintln(os.Stderr, "list containers:", err)
		os.Exit(1)
	}
	for _, c := range byImage(result.Items) {
		fmt.Printf("%4d  %s\n", c.Containers, c.Image)
	}
}
