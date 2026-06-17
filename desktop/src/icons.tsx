// 极简线性图标，统一 16px viewBox，currentColor 描边，匹配 macOS SF Symbols 的轻量感。

type IconProps = { size?: number };

const base = (size: number) => ({
  width: size,
  height: size,
  viewBox: "0 0 16 16",
  fill: "none",
  stroke: "currentColor",
  "stroke-width": 1.5,
  "stroke-linecap": "round" as const,
  "stroke-linejoin": "round" as const,
});

export const CheckIcon = ({ size = 15 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M3.5 8.5l3 3 6-7" />
  </svg>
);

export const ArchiveIcon = ({ size = 15 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M2.5 4.5h11M3.5 4.5l.7 8.2a1 1 0 0 0 1 .8h5.6a1 1 0 0 0 1-.8l.7-8.2M6.3 7.3h3.4" />
  </svg>
);

export const EditIcon = ({ size = 15 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M11.2 2.8l2 2-7.4 7.4-2.6.6.6-2.6 7.4-7.4zM10 4l2 2" />
  </svg>
);

export const LinkIcon = ({ size = 12 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M6.5 9.5l3-3M7 4.5l1-1a2.1 2.1 0 0 1 3 3l-1 1M9 11.5l-1 1a2.1 2.1 0 0 1-3-3l1-1" />
  </svg>
);

export const PlusIcon = ({ size = 16 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M8 3.2v9.6M3.2 8h9.6" />
  </svg>
);

export const TrashIcon = ({ size = 13 }: IconProps) => (
  <svg {...base(size)}>
    <path d="M2.5 4.5h11M5.5 4.5V3.2a.8.8 0 0 1 .8-.8h3.4a.8.8 0 0 1 .8.8v1.3M4 4.5l.6 8.3a1 1 0 0 0 1 .9h4.8a1 1 0 0 0 1-.9l.6-8.3" />
  </svg>
);

export const GearIcon = ({ size = 15 }: IconProps) => (
  <svg {...base(size)}>
    <circle cx="8" cy="8" r="2.1" />
    <path d="M8 1.6v1.6M8 12.8v1.6M14.4 8h-1.6M3.2 8H1.6M12.5 3.5l-1.1 1.1M4.6 11.4l-1.1 1.1M12.5 12.5l-1.1-1.1M4.6 4.6L3.5 3.5" />
  </svg>
);
