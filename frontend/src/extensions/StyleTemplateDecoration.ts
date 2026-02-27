import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    styleTemplate: {
      setStyleTemplate: (prefix: string, suffix: string) => ReturnType;
    };
  }
}

interface StyleTemplateState {
  prefix: string;
  suffix: string;
}

const pluginKey = new PluginKey<StyleTemplateState>('styleTemplate');

function createWidget(text: string): HTMLSpanElement {
  const span = document.createElement('span');
  span.className = 'style-fragment';
  span.textContent = text;
  span.contentEditable = 'false';
  return span;
}

export const StyleTemplateDecoration = Extension.create({
  name: 'styleTemplate',

  addCommands() {
    return {
      setStyleTemplate:
        (prefix: string, suffix: string) =>
        ({ tr, dispatch }) => {
          if (dispatch) {
            tr.setMeta(pluginKey, { prefix, suffix });
            dispatch(tr);
          }
          return true;
        },
    };
  },

  addProseMirrorPlugins() {
    return [
      new Plugin<StyleTemplateState>({
        key: pluginKey,

        state: {
          init(): StyleTemplateState {
            return { prefix: '', suffix: '' };
          },

          apply(tr, value): StyleTemplateState {
            const meta = tr.getMeta(pluginKey) as StyleTemplateState | undefined;
            if (meta) return meta;
            return value;
          },
        },

        props: {
          decorations(state) {
            const { prefix, suffix } = pluginKey.getState(state) ?? { prefix: '', suffix: '' };
            if (!prefix && !suffix) return DecorationSet.empty;

            const decorations: Decoration[] = [];
            const doc = state.doc;

            // Find the first text block (paragraph) to attach decorations
            let contentStart = -1;
            let contentEnd = -1;

            doc.descendants((node, pos) => {
              if (contentStart === -1 && node.isTextblock) {
                // Position inside the textblock: pos is before the node,
                // pos + 1 is the start of its content
                contentStart = pos + 1;
                contentEnd = pos + node.nodeSize - 1;
                return false; // stop traversal
              }
              return true;
            });

            if (contentStart === -1) return DecorationSet.empty;

            if (prefix) {
              decorations.push(
                Decoration.widget(contentStart, () => createWidget(prefix), {
                  side: -1,
                  key: `style-prefix:${prefix}`,
                }),
              );
            }

            if (suffix) {
              decorations.push(
                Decoration.widget(contentEnd, () => createWidget(suffix), {
                  side: 1,
                  key: `style-suffix:${suffix}`,
                }),
              );
            }

            return DecorationSet.create(doc, decorations);
          },
        },
      }),
    ];
  },
});
