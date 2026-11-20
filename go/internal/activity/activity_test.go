// activity: commits-per-author summary used by the Activity Report workflow
package activity

import (
	"strings"
	"testing"
	"time"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSummarizeCountsAndSorts(t *testing.T) {
	s, err := Summarize(strings.NewReader("bob\nalice\nbob\n\ncarol\nalice\nbob\n"))
	require.NoError(t, err)
	assert.Equal(t, 6, s.Total)
	assert.Equal(t, []Author{{"bob", 3}, {"alice", 2}, {"carol", 1}}, s.Authors)
}

func TestSummarizeEmpty(t *testing.T) {
	s, err := Summarize(strings.NewReader(""))
	require.NoError(t, err)
	assert.Zero(t, s.Total)
	assert.Empty(t, s.Authors)
}

func TestFromRepoCountsRecentCommits(t *testing.T) {
	dir := t.TempDir()
	repo, err := git.PlainInit(dir, false)
	require.NoError(t, err)
	wt, err := repo.Worktree()
	require.NoError(t, err)

	commit := func(name string, when time.Time) {
		_, err := wt.Commit("change", &git.CommitOptions{
			AllowEmptyCommits: true,
			Author:            &object.Signature{Name: name, Email: name + "@example.com", When: when},
		})
		require.NoError(t, err)
	}
	now := time.Now()
	commit("old", now.Add(-72*time.Hour))
	commit("alice", now.Add(-2*time.Hour))
	commit("alice", now.Add(-time.Hour))

	s, err := FromRepo(dir, now.Add(-24*time.Hour))
	require.NoError(t, err)
	assert.Equal(t, Summary{Total: 2, Authors: []Author{{"alice", 2}}}, s)
}

func TestFromRepoRejectsNonRepository(t *testing.T) {
	_, err := FromRepo(t.TempDir(), time.Now())
	assert.Error(t, err)
}
