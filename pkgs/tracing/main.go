// Command tracing is the reference OpenTelemetry setup for the platform's Go
// services: it runs a traced batch job (aggregating counts in batches, like the
// nightly rollup) and exports spans over OTLP/gRPC (OTEL_EXPORTER_OTLP_ENDPOINT)
// or to stdout with -stdout.
package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/exporters/stdout/stdouttrace"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"go.opentelemetry.io/otel/trace"
)

// work sums 1..n inside a span, recording one child span per batch of 10.
func work(ctx context.Context, tracer trace.Tracer, n int) int {
	ctx, span := tracer.Start(ctx, "work", trace.WithAttributes(attribute.Int("n", n)))
	defer span.End()
	total := 0
	for start := 1; start <= n; start += 10 {
		_, batch := tracer.Start(ctx, "batch", trace.WithAttributes(attribute.Int("start", start)))
		for i := start; i < start+10 && i <= n; i++ {
			total += i
		}
		batch.End()
	}
	span.SetAttributes(attribute.Int("total", total))
	return total
}

func main() {
	n := flag.Int("n", 100, "sum 1..n")
	toStdout := flag.Bool("stdout", false, "print spans instead of exporting over OTLP")
	flag.Parse()
	ctx := context.Background()

	var exporter sdktrace.SpanExporter
	var err error
	if *toStdout {
		exporter, err = stdouttrace.New(stdouttrace.WithPrettyPrint())
	} else {
		exporter, err = otlptracegrpc.New(ctx)
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, "create exporter:", err)
		os.Exit(1)
	}
	provider := sdktrace.NewTracerProvider(sdktrace.WithBatcher(exporter))
	defer provider.Shutdown(ctx)
	otel.SetTracerProvider(provider)

	fmt.Println(work(ctx, otel.Tracer("tracing"), *n))
}
