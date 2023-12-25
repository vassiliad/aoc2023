import networkx as nx

# Put this together while waiting for the rust implementation to finish running
# it uses the Stoer Wagner algorithm from networkx

sample = """jqt: rhn xhk nvd
rsh: frs pzl lsr
xhk: hfx
cmg: qnr nvd lhk bvb
rhn: xhk bvb hfx
bvb: xhk hfx
pzl: lsr hfx nvd
qnr: nvd
ntq: jqt hfx bvb xhk
nvd: lhk
lsr: lhk
rzs: qnr cmg lsr rsh
frs: qnr lhk lsr
"""

sample = open("../input/mine", "r").read()

G = nx.Graph()

for line in sample.strip().split("\n"):
    left, rest = line.split(":", 1)
    left = left.strip()
    rest = rest.split(" ")
    for right in rest:
        if not right:
            continue
        G.add_edge(left, right, weight=1)

cut_value, partition = nx.stoer_wagner(G)

left, right = partition

print(len(left)*len(right))
