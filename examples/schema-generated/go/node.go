package schema

import "encoding/json"

type Node struct {
    Id string `json:"id"`
    Title string `json:"title"`
    Position [4]float64 `json:"position"`
}

type HyperEdge struct {
    Id string `json:"id"`
    Nodes []string `json:"nodes"`
    Label string `json:"label,omitempty"`
}

