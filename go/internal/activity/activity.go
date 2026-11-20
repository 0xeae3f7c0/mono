// activity: commits-per-author summary used by the Activity Report workflow
// Package activity summarizes commit activity from `git log` output.
package activity

import (
	"bufio"
	"io"
	"sort"
	"strings"
)

// Author is one author's commit count.
type Author struct {
	Name    string `json:"name"`
	Commits int    `json:"commits"`
}

// Summary is the total plus per-author counts, busiest first.
type Summary struct {
	Total   int      `json:"total"`
	Authors []Author `json:"authors"`
}

// Summarize reads one author name per line (git log --format=%an).
func Summarize(r io.Reader) (Summary, error) {
	counts := map[string]int{}
	scanner := bufio.NewScanner(r)
	for scanner.Scan() {
		name := strings.TrimSpace(scanner.Text())
		if name != "" {
			counts[name]++
		}
	}
	if err := scanner.Err(); err != nil {
		return Summary{}, err
	}

	s := Summary{}
	for name, n := range counts {
		s.Total += n
		s.Authors = append(s.Authors, Author{Name: name, Commits: n})
	}
	sort.Slice(s.Authors, func(i, j int) bool {
		if s.Authors[i].Commits != s.Authors[j].Commits {
			return s.Authors[i].Commits > s.Authors[j].Commits
		}
		return s.Authors[i].Name < s.Authors[j].Name
	})
	return s, nil
}
