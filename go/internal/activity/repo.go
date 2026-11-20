package activity

import (
	"bytes"
	"fmt"
	"time"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing/object"
)

// FromRepo summarizes commits on HEAD of the repository at path made after since.
func FromRepo(path string, since time.Time) (Summary, error) {
	repo, err := git.PlainOpenWithOptions(path, &git.PlainOpenOptions{DetectDotGit: true})
	if err != nil {
		return Summary{}, fmt.Errorf("open repository: %w", err)
	}
	iter, err := repo.Log(&git.LogOptions{Since: &since})
	if err != nil {
		return Summary{}, fmt.Errorf("read log: %w", err)
	}
	defer iter.Close()

	var names bytes.Buffer
	err = iter.ForEach(func(c *object.Commit) error {
		names.WriteString(c.Author.Name + "\n")
		return nil
	})
	if err != nil {
		return Summary{}, err
	}
	return Summarize(&names)
}
