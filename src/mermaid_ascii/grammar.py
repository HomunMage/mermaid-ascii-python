"""PEG grammar for Mermaid flowchart syntax.

Translated from grammar.pest (pest/Rust) to parsimonious (Python) format.

In pest, WHITESPACE is auto-inserted between tokens in non-atomic rules.
In parsimonious, whitespace must be explicit. Strategy:
  - OWS (optional whitespace) is inserted around connectors and between tokens
  - NEWLINE consumes trailing comments and the actual newline char
  - Line-level rules (statement, etc.) begin with OWS to consume leading indent
  - EOI is a regex matching end-of-string (with optional trailing whitespace)

This grammar is used directly by parser.py via parsimonious.
"""

GRAMMAR = r"""
file            = header? statement* EOI

header          = graph_keyword OWS direction_value NEWLINE
graph_keyword   = "flowchart" / "graph"
direction_value = "TD" / "TB" / "LR" / "RL" / "BT"

statement       = OWS (subgraph_block / edge_stmt / node_stmt / blank_line)

blank_line      = NEWLINE

node_ref        = node_id node_shape?
node_id         = ~"[A-Za-z_][A-Za-z0-9_-]*"
node_shape      = circle_shape / rounded_shape / diamond_shape / rect_shape

rect_shape      = "[" node_label "]"
rounded_shape   = "(" !"(" node_label ")"
diamond_shape   = "{" node_label "}"
circle_shape    = "((" node_label "))"

node_label      = quoted_string / unquoted_label
unquoted_label  = ~r"(?:[^\]\)\}\n\r])+"

node_stmt       = node_ref OWS NEWLINE

edge_stmt       = node_ref edge_chain OWS NEWLINE
edge_chain      = edge_segment+
edge_segment    = OWS edge_connector OWS edge_label? node_ref

edge_label      = "|" label_text "|" OWS
label_text      = ~r"[^|\n\r]+"

edge_connector  = bidir_dotted / bidir_thick / bidir_arrow / dotted_arrow / thick_arrow / arrow / dotted_line / thick_line / line

arrow           = "-->"
line            = "---"
dotted_arrow    = "-.->"
dotted_line     = "-.-"
thick_arrow     = "==>"
thick_line      = "==="
bidir_arrow     = "<-->"
bidir_dotted    = "<-.->"
bidir_thick     = "<==>"

subgraph_block     = subgraph_keyword OWS subgraph_label NEWLINE subgraph_direction? subgraph_body OWS end_keyword OWS NEWLINE?
subgraph_keyword   = "subgraph"
subgraph_direction = OWS "direction" OWS direction_value NEWLINE
subgraph_label     = quoted_string / bare_subgraph_label
bare_subgraph_label = ~r"[^\n\r]+"
subgraph_body      = (!end_kw_lookahead statement)*
end_kw_lookahead   = OWS end_keyword
end_keyword        = ~r"end(?![A-Za-z0-9_-])"

quoted_string      = ~r'"(?:[^"\\]|\\.)*"'

NEWLINE   = ~r"[ \t]*(%%[^\n\r]*)?(\r\n|\n|\r)"
OWS       = ~r"[ \t]*"
EOI       = ~r"[ \t\r\n]*$"
"""
