export interface Node {
  id: string;
  title: string;
  position: [number,number,number,number];
}

export interface HyperEdge {
  id: string;
  nodes: string[];
  label?: string;
}

