import type { Plugin } from 'vue';
import * as Lucide from 'lucide-vue-next';

// Tag is only registered as 'TagIcon' alias to avoid collision with PrimeVue's Tag component

const iconNames = [
  'AlertCircle', 'ArrowDownAZ', 'ArrowLeft', 'ArrowLeftToLine', 'ArrowRight',
  'ArrowUpAZ', 'ArrowUpDown', 'Bot', 'Box', 'Brain', 'Check', 'ChevronDown',
  'ChevronRight', 'ClipboardCheck', 'Cloud', 'Cog', 'Copy', 'Cpu', 'Database',
  'Download', 'Eye', 'EyeOff', 'FolderInput', 'FolderOpen', 'Globe', 'Gpu',
  'Grid3x3', 'HardDrive', 'ImageOff', 'ImagePlus', 'Images', 'Info',
  'KeyRound', 'KeySquare', 'Layers', 'LayoutGrid', 'ListTodo', 'Lock',
  'MessageCirclePlus', 'Paintbrush', 'Palette', 'PenTool', 'Play', 'Plus',
  'Save', 'Search', 'SendHorizontal', 'Shield', 'ShieldAlert', 'Shuffle', 'SlidersHorizontal',
  'Sparkles', 'Square', 'Star', 'Table', 'Terminal', 'Trash2',
  'TriangleAlert', 'Unlink', 'Upload', 'WandSparkles', 'Wifi', 'X', 'Server',
  'ServerOff',
] as const;

// Aliases used by components that imported with renames
const aliases: Record<string, keyof typeof Lucide> = {
  ImageIcon: 'Image',
  FolderIcon: 'Folder',
  ListIcon: 'List',
  TagIcon: 'Tag',
};

export const LucidePlugin: Plugin = {
  install(app) {
    for (const name of iconNames) {
      app.component(name, Lucide[name]);
    }
    for (const [alias, name] of Object.entries(aliases)) {
      app.component(alias, Lucide[name]);
    }
  },
};