export interface KeyInfo {
  key: string;
  ttl: number;
}

export interface TreeNode {
  name: string;
  path: string;
  children: TreeNode[];
  isLeaf: boolean;
  count: number;
  ttl?: number;
}

// ponytail: internal build node uses Record for children so lookups are O(1)
// and we don't lose children when converting between map/array
interface BuildNode {
  name: string;
  path: string;
  children: Record<string, BuildNode>;
  isKey: boolean;
  ttl?: number;
}

export function buildTree(items: KeyInfo[], separator: string = ":"): TreeNode[] {
  const root: Record<string, BuildNode> = {};

  for (const item of items) {
    const parts = item.key.split(separator).filter(p => p.length > 0);
    let current = root;
    let path = "";

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      path = path ? `${path}${separator}${part}` : part;
      const isLast = i === parts.length - 1;

      if (!current[part]) {
        current[part] = {
          name: part,
          path,
          children: {},
          isKey: false,
        };
      }

      if (isLast) {
        current[part].isKey = true;
        current[part].path = item.key;
        current[part].ttl = item.ttl;
      }

      current = current[part].children;
    }
  }

  return sortNodes(toTreeNodes(root));
}

function toTreeNodes(record: Record<string, BuildNode>): TreeNode[] {
  return Object.values(record).map(node => {
    let children = toTreeNodes(node.children);
    if (node.isKey && children.length > 0) {
      children = [
        {
          name: "(value)",
          path: node.path,
          children: [],
          isLeaf: true,
          count: 1,
          ttl: node.ttl,
        },
        ...children
      ];
    }
    const isLeaf = children.length === 0;
    const count = isLeaf ? 1 : children.reduce((acc, c) => acc + c.count, 0);

    return {
      name: node.name,
      path: node.path,
      children,
      isLeaf,
      count,
      ttl: node.ttl,
    };
  });
}

function sortNodes(nodes: TreeNode[]): TreeNode[] {
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
