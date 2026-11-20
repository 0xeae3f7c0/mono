package main

import (
	"context"
	"testing"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/service/s3"
	"github.com/aws/aws-sdk-go-v2/service/s3/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type fakePages []*s3.ListObjectsV2Output

func (f *fakePages) ListObjectsV2(context.Context, *s3.ListObjectsV2Input, ...func(*s3.Options)) (*s3.ListObjectsV2Output, error) {
	page := (*f)[0]
	*f = (*f)[1:]
	return page, nil
}

func obj(size int64) types.Object { return types.Object{Size: aws.Int64(size)} }

func TestTotalsAcrossPages(t *testing.T) {
	pages := fakePages{
		{Contents: []types.Object{obj(10), obj(5)}, IsTruncated: aws.Bool(true), NextContinuationToken: aws.String("t")},
		{Contents: []types.Object{obj(1)}, IsTruncated: aws.Bool(false)},
	}
	n, size, err := totals(context.Background(), &pages, "b", "")
	require.NoError(t, err)
	assert.Equal(t, 3, n)
	assert.Equal(t, int64(16), size)
}
