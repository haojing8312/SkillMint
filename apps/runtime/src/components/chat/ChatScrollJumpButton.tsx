import { motion } from "framer-motion";

interface ChatScrollJumpButtonProps {
  visible: boolean;
  isNearBottom: boolean;
  label: string;
  hint: string;
  onClick: () => void;
}

export function ChatScrollJumpButton({
  visible,
  isNearBottom,
  label,
  hint,
  onClick,
}: ChatScrollJumpButtonProps) {
  if (!visible) {
    return null;
  }

  return (
    <div className="pointer-events-none absolute inset-x-0 bottom-5 z-20 flex justify-center">
      <motion.button
        type="button"
        data-testid="chat-scroll-jump-button"
        aria-label={label}
        title={hint}
        onClick={onClick}
        initial={false}
        animate={{
          opacity: isNearBottom ? 0.94 : 0.88,
          y: isNearBottom ? 0 : -20,
          scale: isNearBottom ? 1 : 0.985,
        }}
        transition={{ type: "spring", stiffness: 240, damping: 28, mass: 0.8 }}
        className="pointer-events-auto flex h-9 w-9 items-center justify-center rounded-full border border-slate-200/85 bg-[#f4f4f1]/92 text-slate-500 shadow-[0_6px_16px_rgba(15,23,42,0.08)] transition-all duration-200 hover:border-slate-300 hover:bg-[#f7f7f4] hover:text-slate-700 hover:shadow-[0_10px_22px_rgba(15,23,42,0.1)]"
      >
        <motion.span
          aria-hidden="true"
          initial={false}
          animate={{ rotate: isNearBottom ? 0 : 180 }}
          transition={{ duration: 0.22, ease: "easeInOut" }}
          className="translate-y-[-1px] text-[20px] leading-none"
        >
          ↑
        </motion.span>
      </motion.button>
    </div>
  );
}
