// activity: commits-per-author summary used by the Activity Report workflow
// Command activity prints commit counts per author for a time window.
//
//	go run ./cmd/activity --since 24h --repo ..
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"time"

	"github.com/spf13/cobra"

	"commit/go/internal/activity"
)

func main() {
	var (
		since  time.Duration
		repo   string
		asJSON bool
	)
	cmd := &cobra.Command{
		Use:          "activity",
		Short:        "Summarize commits per author over a time window",
		Args:         cobra.NoArgs,
		SilenceUsage: true,
		RunE: func(cmd *cobra.Command, _ []string) error {
			s, err := activity.FromRepo(repo, time.Now().Add(-since))
			if err != nil {
				return err
			}
			out := cmd.OutOrStdout()
			if asJSON {
				enc := json.NewEncoder(out)
				enc.SetIndent("", "  ")
				return enc.Encode(s)
			}
			fmt.Fprintf(out, "%d commits in the last %s\n", s.Total, since)
			for _, a := range s.Authors {
				fmt.Fprintf(out, "  %4d  %s\n", a.Commits, a.Name)
			}
			return nil
		},
	}
	cmd.Flags().DurationVar(&since, "since", 24*time.Hour, "time window to summarize")
	cmd.Flags().StringVar(&repo, "repo", ".", "repository path")
	cmd.Flags().BoolVar(&asJSON, "json", false, "print JSON")

	if err := cmd.Execute(); err != nil {
		os.Exit(1)
	}
}
