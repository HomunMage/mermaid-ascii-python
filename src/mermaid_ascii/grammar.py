"""
PEG grammar for Mermaid flowchart syntax.

Translated from grammar.pest (pest/Rust) to parsimonious (Python) format.

Key translation notes from pest → parsimonious:
  - pest `~` (sequence)  → parsimonious space between expressions
  - pest `|` (ordered choice) → parsimonious `/`
  - pest `@{ }` (atomic, no internal whitespace) → inline regex or careful rule structure
  - pest `_{ }` (silent rule) → still present in tree but ignored in visitor
  - pest `WHITESPACE = _{}` auto-insertion → parsimonious requires explicit `_` whitespace rule
  - parsimonious uses `~r"..."` for regex literals
"""

GRAMMAR = r"""
file          = header? statement* EOI

header        = graph_keyword WS direction_value NEWLINE
graph_keyword = "flowchart" / "graph"
direction_value = "TD" / "TB" / "LR" / "RL" / "BT"

statement     = subgraph_block / edge_stmt / node_stmt / blank_line

blank_line    = NEWLINE

node_ref      = node_id node_shape?
node_id       = ~r"[A-Za-z_][A-Za-z0-9_-]*"
node_shape    = circle_shape / rounded_shape / diamond_shape / rect_shape

rect_shape    = "[" node_label "]"
rounded_shape = "(" !"(" node_label ")"
diamond_shape = "{" node_label "}"
circle_shape  = "((" node_label "))"

node_label    = quoted_string / unquoted_label
unquoted_label = ~r"(?:[^\])\}\n\r]|(?<!\))\)(?!\)))+"

node_stmt     = node_ref NEWLINE

edge_stmt     = node_ref edge_chain NEWLINE
edge_chain    = edge_segment+
edge_segment  = edge_connector WS? edge_label? WS? node_ref

edge_label    = "|" label_text "|"
label_text    = ~r"[^|\n\r]+"

edge_connector = bidir_dotted / bidir_thick / bidir_arrow / dotted_arrow / thick_arrow / arrow / dotted_line / thick_line / line

arrow         = "-->"
line          = "---"
dotted_arrow  = ".->"
dotted_line   = "-.-"
thick_arrow   = "==>"
thick_line    = "==="
bidir_arrow   = "<-->"
bidir_dotted  = "<-.->"
bidir_thick   = "<==>"

subgraph_block     = subgraph_keyword WS subgraph_label NEWLINE subgraph_direction? subgraph_body end_keyword NEWLINE?
subgraph_keyword   = "subgraph"
subgraph_direction = "direction" WS direction_value NEWLINE
subgraph_label     = quoted_string / bare_subgraph_label
bare_subgraph_label = ~r"[^\n\r]+"
subgraph_body      = (!end_keyword statement)*
end_keyword        = ~r"end(?![A-Za-z0-9_-])"

quoted_string = ~r'"(?:[^"\\]|\\.)*"'

WS            = ~r"[ \t]+"
OWS           = ~r"[ \t]*"
NEWLINE       = ~r"(\r\n|\n|\r)([ \t]*(%%[^\n\r]*)?\s*)*"
COMMENT       = ~r"%%[^\n\r]*"
EOI           = ~r"$"
"""
