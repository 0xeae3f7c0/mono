// pipelines: Weekly report workflow (commit platform, pkgs/pipelines)
// Command pipelines runs the Temporal worker for the weekly report pipeline:
// the ReportWorkflow builds a repository's report title and summary line via
// an activity (retried by Temporal), before reports-pdf renders and archive
// uploads it. Requires a Temporal server (TEMPORAL_ADDRESS, default localhost:7233).
package main

import (
	"context"
	"fmt"
	"os"
	"time"

	"go.temporal.io/sdk/client"
	"go.temporal.io/sdk/worker"
	"go.temporal.io/sdk/workflow"
)

const taskQueue = "reports"

// ReportRequest identifies the repository and ISO week to report on.
type ReportRequest struct {
	Repo string
	Week string
}

// BuildHeadline is the activity: it produces the report headline.
func BuildHeadline(_ context.Context, req ReportRequest) (string, error) {
	if req.Repo == "" || req.Week == "" {
		return "", fmt.Errorf("repo and week are required")
	}
	return fmt.Sprintf("Weekly report: %s (%s)", req.Repo, req.Week), nil
}

// ReportWorkflow runs BuildHeadline with a timeout and returns its result.
func ReportWorkflow(ctx workflow.Context, req ReportRequest) (string, error) {
	ctx = workflow.WithActivityOptions(ctx, workflow.ActivityOptions{StartToCloseTimeout: 10 * time.Second})
	var headline string
	err := workflow.ExecuteActivity(ctx, BuildHeadline, req).Get(ctx, &headline)
	return headline, err
}

func main() {
	c, err := client.Dial(client.Options{HostPort: os.Getenv("TEMPORAL_ADDRESS")})
	if err != nil {
		fmt.Fprintln(os.Stderr, "connect to Temporal:", err)
		os.Exit(1)
	}
	defer c.Close()
	w := worker.New(c, taskQueue, worker.Options{})
	w.RegisterWorkflow(ReportWorkflow)
	w.RegisterActivity(BuildHeadline)
	if err := w.Run(worker.InterruptCh()); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
