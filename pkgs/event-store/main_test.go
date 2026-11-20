package main

import (
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestAddAndList(t *testing.T) {
	db, err := open(filepath.Join(t.TempDir(), "events.db"))
	require.NoError(t, err)
	sqlDB, err := db.DB()
	require.NoError(t, err)
	t.Cleanup(func() { sqlDB.Close() }) // Windows cannot delete an open file

	_, err = add(db, "first")
	require.NoError(t, err)
	_, err = add(db, "second")
	require.NoError(t, err)

	tasks, err := list(db)
	require.NoError(t, err)
	assert.Equal(t, []Event{{ID: 1, Title: "first"}, {ID: 2, Title: "second"}}, tasks)
}
