// pipelines: Weekly report workflow (commit platform, pkgs/pipelines)
package main

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.temporal.io/sdk/testsuite"
)

func TestReportWorkflow(t *testing.T) {
	var suite testsuite.WorkflowTestSuite
	env := suite.NewTestWorkflowEnvironment()
	env.RegisterActivity(BuildHeadline)

	env.ExecuteWorkflow(ReportWorkflow, ReportRequest{Repo: "o/r", Week: "2026-W47"})

	require.True(t, env.IsWorkflowCompleted())
	require.NoError(t, env.GetWorkflowError())
	var headline string
	require.NoError(t, env.GetWorkflowResult(&headline))
	assert.Equal(t, "Weekly report: o/r (2026-W47)", headline)
}
