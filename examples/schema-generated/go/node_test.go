package schema

import (
	"encoding/json"
	"testing"
)

func TestNodeRoundtrip(t *testing.T) {
	n := Node{Id: "1111-2222", Title: "Alice", Position: [4]float64{1, 2, 3, 4}}
	b, err := SerializeNode(n)
	if err != nil {
		t.Fatalf("serialize error: %v", err)
	}
	var n2 Node
	err = json.Unmarshal(b, &n2)
	if err != nil {
		t.Fatalf("deserialize error: %v", err)
	}
	if n.Id != n2.Id {
		t.Fatalf("id mismatch")
	}
}

func TestHyperEdgeRoundtrip(t *testing.T) {
	he := HyperEdge{Id: "he1", Nodes: []string{"1111-2222"}, Label: nil}
	b, err := json.Marshal(he)
	if err != nil {
		t.Fatalf("serialize error: %v", err)
	}
	var he2 HyperEdge
	err = json.Unmarshal(b, &he2)
	if err != nil {
		t.Fatalf("deserialize error: %v", err)
	}
	if he.Id != he2.Id {
		t.Fatalf("id mismatch")
	}
}
