// Command runner-fleet reports the self-hosted CI runner fleet on Kubernetes:
// pod counts per namespace and phase, so stuck or pending runners stand out.
package main

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"sort"

	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/tools/clientcmd"
)

// Count is the number of pods in one namespace and phase.
type Count struct {
	Namespace, Phase string
	Pods             int
}

func report(ctx context.Context, client kubernetes.Interface) ([]Count, error) {
	pods, err := client.CoreV1().Pods("").List(ctx, metav1.ListOptions{})
	if err != nil {
		return nil, err
	}
	counts := map[[2]string]int{}
	for _, p := range pods.Items {
		counts[[2]string{p.Namespace, string(p.Status.Phase)}]++
	}
	out := make([]Count, 0, len(counts))
	for k, n := range counts {
		out = append(out, Count{k[0], k[1], n})
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Namespace != out[j].Namespace {
			return out[i].Namespace < out[j].Namespace
		}
		return out[i].Phase < out[j].Phase
	})
	return out, nil
}

func main() {
	kubeconfig := os.Getenv("KUBECONFIG")
	if kubeconfig == "" {
		home, _ := os.UserHomeDir()
		kubeconfig = filepath.Join(home, ".kube", "config")
	}
	config, err := clientcmd.BuildConfigFromFlags("", kubeconfig)
	if err != nil {
		fmt.Fprintln(os.Stderr, "load kubeconfig:", err)
		os.Exit(1)
	}
	counts, err := report(context.Background(), kubernetes.NewForConfigOrDie(config))
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	for _, c := range counts {
		fmt.Printf("%-24s %-10s %d\n", c.Namespace, c.Phase, c.Pods)
	}
}
