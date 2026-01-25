export const panel = {
  root: {
    borderRadius: '0rem',
  },
  content: {
    padding: '0rem',
  },
  footer: {
    padding: '0rem',
  },
  header: {
    padding: '0rem',
    borderWidth: '0rem',
    borderRadius: '0rem',
  },
  title: {
    fontWeight: '600',
  },
  toggleableHeader: {
    padding: '0rem',
  },
  colorScheme: {
    light: {
      root: {
        background: '{surface.800}',
        borderColor: '{surface.800}',
        color: '{content.color}',
      },
      content: {},
      footer: {},
      header: {
        background: 'transparent',
        color: '{text.color}',
        borderColor: '{surface.800}',
      },
      title: {},
      toggleableHeader: {},
    },
    dark: {
      root: {
        background: 'transparent',
        borderColor: 'transparent',
        color: '{content.color}',
      },
      content: {},
      footer: {},
      header: {
        background: 'transparent',
        color: '{text.color}',
        borderColor: '{content.border.color}',
      },
      title: {},
      toggleableHeader: {},
    },
  },
};
