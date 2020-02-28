package influxdb_test

import (
	"testing"
	"time"

	"github.com/influxdata/flux"
	"github.com/influxdata/flux/execute"
	"github.com/influxdata/flux/querytest"
	"github.com/influxdata/flux/stdlib/influxdata/influxdb"
	"github.com/influxdata/flux/stdlib/universe"
)

func TestFrom_NewQuery(t *testing.T) {
	tests := []querytest.NewQueryTestCase{
		{
			Name:    "from no args",
			Raw:     `from()`,
			WantErr: true,
		},
		{
			Name:    "from unexpected arg",
			Raw:     `from(bucket:"telegraf", chicken:"what is this?")`,
			WantErr: true,
		},
		{
			Name: "from with database",
			Raw:  `from(bucket:"mybucket") |> range(start:-4h, stop:-2h) |> sum()`,
			Want: &flux.Spec{
				Operations: []*flux.Operation{
					{
						ID: "from0",
						Spec: &influxdb.FromOpSpec{
							Bucket: influxdb.NameOrID{Name: "mybucket"},
						},
					},
					{
						ID: "range1",
						Spec: &universe.RangeOpSpec{
							Start: flux.Time{
								Relative:   -4 * time.Hour,
								IsRelative: true,
							},
							Stop: flux.Time{
								Relative:   -2 * time.Hour,
								IsRelative: true,
							},
							TimeColumn:  "_time",
							StartColumn: "_start",
							StopColumn:  "_stop",
						},
					},
					{
						ID: "sum2",
						Spec: &universe.SumOpSpec{
							AggregateConfig: execute.DefaultAggregateConfig,
						},
					},
				},
				Edges: []flux.Edge{
					{Parent: "from0", Child: "range1"},
					{Parent: "range1", Child: "sum2"},
				},
			},
		},
		{
			Name: "from with host and token",
			Raw:  `from(bucket:"mybucket", host: "http://localhost:9999", token: "mytoken")`,
			Want: &flux.Spec{
				Operations: []*flux.Operation{
					{
						ID: "from0",
						Spec: &influxdb.FromOpSpec{
							Bucket: influxdb.NameOrID{Name: "mybucket"},
							Host:   stringPtr("http://localhost:9999"),
							Token:  stringPtr("mytoken"),
						},
					},
				},
			},
		},
		{
			Name: "from with org",
			Raw:  `from(org: "influxdata", bucket:"mybucket")`,
			Want: &flux.Spec{
				Operations: []*flux.Operation{
					{
						ID: "from0",
						Spec: &influxdb.FromOpSpec{
							Org:    &influxdb.NameOrID{Name: "influxdata"},
							Bucket: influxdb.NameOrID{Name: "mybucket"},
						},
					},
				},
			},
		},
		{
			Name: "from with org id and bucket id",
			Raw:  `from(orgID: "97aa81cc0e247dc4", bucketID: "1e01ac57da723035")`,
			Want: &flux.Spec{
				Operations: []*flux.Operation{
					{
						ID: "from0",
						Spec: &influxdb.FromOpSpec{
							Org:    &influxdb.NameOrID{ID: "97aa81cc0e247dc4"},
							Bucket: influxdb.NameOrID{ID: "1e01ac57da723035"},
						},
					},
				},
			},
		},
	}

	for _, tc := range tests {
		tc := tc
		t.Run(tc.Name, func(t *testing.T) {
			t.Parallel()
			querytest.NewQueryTestHelper(t, tc)
		})
	}
}

func stringPtr(v string) *string {
	return &v
}
