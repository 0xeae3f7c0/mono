package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestRenderExampleChart(t *testing.T) {
	out, err := render("testdata/web", "demo")
	require.NoError(t, err)
	require.Len(t, out, 1)
	manifest := out["web/templates/configmap.yaml"]
	assert.Contains(t, manifest, "name: demo-web")
	assert.Contains(t, manifest, `greeting: "hello"`)
}

func TestRenderMissingChart(t *testing.T) {
	_, err := render("testdata/missing", "demo")
	assert.Error(t, err)
}

func TestRenderPlatformChart(t *testing.T) {
	out, err := render("../../deploy/helm/commit", "prod")
	require.NoError(t, err)
	manifest := out["commit/templates/services.yaml"]
	for _, svc := range []string{"events-api", "webhooks", "badges"} {
		assert.Contains(t, manifest, "name: prod-"+svc)
	}
	assert.Contains(t, manifest, "replicas: 2")
}
