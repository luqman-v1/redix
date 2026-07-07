export interface TreeNode {
  name: string;
  path: string;
  children: TreeNode[];
  isLeaf: boolean;
  count: number;
}

export function buildTree(keys: string[], separator: string = ":"): TreeNode[] {
  const root: Record<string, TreeNode> = {};

  for (const key of keys) {
    const parts = key.split(separator).filter(p => p.length > 0);
    let current = root;
    let path = "";

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      path = path ? `${path}${separator}${part}` : part;
      const isLeaf = i === parts.length - 1;

      if (!current[part]) {
        current[part] = {
          name: part,
          path,
          children: [],
          isLeaf,
          count: 0,
        };
      }

      if (isLeaf) {
        current[part].isLeaf = true;
        current[part].path = key;
      }

      // pivot: if node was leaf but now has children, make it folder
      if (!isLeaf && current[part].isLeaf) {
        current[part].isLeaf = false;
      }

      if (!isLeaf) {
        // build children index from existing children array
        const childIndex: Record<string, TreeNode> = {};
        for (const child of current[part].children) {
          childIndex[child.name] = child;
        }
        current = childIndex;
      }
    }
  }

  return sortNodes(Object.values(root));
}

function sortNodes(nodes: TreeNode[]): TreeNode[] {
  // count leaf descendants, sort children recursively
  for (const node of nodes) {
    node.children = sortNodes(node.children);
    node.count = node.isLeaf
      ? 1
      : node.children.reduce((sum, c) => sum + c.count, 0);
  }

  // folders first, then leaves, both alphabetical
  return [...nodes].sort((a, b) => {
    if (a.isLeaf !== b.isLeaf) return a.isLeaf ? 1 : -1;
    return a.name.localeCompare(b.name);
  });
}
