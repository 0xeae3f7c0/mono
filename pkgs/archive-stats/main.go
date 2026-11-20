// archive-stats: Reports archive bucket usage (commit platform, pkgs/archive-stats)
// Command archive-stats reports how many archived reports (objects) and bytes
// sit under a prefix in the report bucket that `archive` uploads to.
package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/s3"
)

// totals walks every page of the listing and returns object count and bytes.
func totals(ctx context.Context, client s3.ListObjectsV2APIClient, bucket, prefix string) (objects int, bytes int64, err error) {
	pages := s3.NewListObjectsV2Paginator(client, &s3.ListObjectsV2Input{Bucket: aws.String(bucket), Prefix: aws.String(prefix)})
	for pages.HasMorePages() {
		page, err := pages.NextPage(ctx)
		if err != nil {
			return 0, 0, err
		}
		for _, obj := range page.Contents {
			objects++
			bytes += aws.ToInt64(obj.Size)
		}
	}
	return objects, bytes, nil
}

func main() {
	bucket := flag.String("bucket", "", "bucket name (required)")
	prefix := flag.String("prefix", "", "key prefix")
	flag.Parse()
	if *bucket == "" {
		flag.Usage()
		os.Exit(2)
	}
	ctx := context.Background()
	cfg, err := config.LoadDefaultConfig(ctx)
	if err != nil {
		fmt.Fprintln(os.Stderr, "load AWS config:", err)
		os.Exit(1)
	}
	n, size, err := totals(ctx, s3.NewFromConfig(cfg), *bucket, *prefix)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	fmt.Printf("%d objects, %d bytes\n", n, size)
}
