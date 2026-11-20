// tracing: Reference OpenTelemetry setup for Go services (commit platform, pkgs/tracing)
package main

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"go.opentelemetry.io/otel/sdk/trace/tracetest"
)

func TestWorkRecordsSpans(t *testing.T) {
	recorder := tracetest.NewSpanRecorder()
	provider := sdktrace.NewTracerProvider(sdktrace.WithSpanProcessor(recorder))

	assert.Equal(t, 300, work(context.Background(), provider.Tracer("test"), 24))

	spans := recorder.Ended()
	assert.Len(t, spans, 4) // 3 batches + root
	assert.Equal(t, "work", spans[len(spans)-1].Name())
}
