// Command helm-render renders the platform's Helm chart (deploy/helm/commit)
// with its default values and prints each manifest, so deployment changes can
// be reviewed as plain YAML in pull requests.
package main

import (
	"fmt"
	"os"
	"sort"

	"helm.sh/helm/v3/pkg/chart/loader"
	"helm.sh/helm/v3/pkg/chartutil"
	"helm.sh/helm/v3/pkg/engine"
)

// render returns manifests keyed by template path, with empty output dropped.
func render(chartDir, releaseName string) (map[string]string, error) {
	chart, err := loader.Load(chartDir)
	if err != nil {
		return nil, fmt.Errorf("load chart: %w", err)
	}
	values, err := chartutil.ToRenderValues(chart, chart.Values,
		chartutil.ReleaseOptions{Name: releaseName, Namespace: "default", IsInstall: true},
		chartutil.DefaultCapabilities)
	if err != nil {
		return nil, err
	}
	out, err := engine.Render(chart, values)
	if err != nil {
		return nil, fmt.Errorf("render: %w", err)
	}
	for name, manifest := range out {
		if len(manifest) == 0 {
			delete(out, name)
		}
	}
	return out, nil
}

func main() {
	if len(os.Args) != 3 {
		fmt.Fprintln(os.Stderr, "usage: helm-render <chart-dir> <release-name>")
		os.Exit(2)
	}
	manifests, err := render(os.Args[1], os.Args[2])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	names := make([]string, 0, len(manifests))
	for name := range manifests {
		names = append(names, name)
	}
	sort.Strings(names)
	for _, name := range names {
		fmt.Printf("---\n# Source: %s\n%s\n", name, manifests[name])
	}
}
