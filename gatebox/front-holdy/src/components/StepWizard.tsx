interface Props {
  steps: string[]
  current: number
}

export function StepWizard({ steps, current }: Props) {
  return (
    <div className="flex items-center gap-0 mb-6">
      {steps.map((label, i) => {
        const done = i < current
        const active = i === current
        return (
          <div key={i} className="flex items-center flex-1">
            <div className="flex flex-col items-center">
              <div
                className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold border-2 transition-colors ${
                  done
                    ? 'bg-brand border-brand text-white'
                    : active
                    ? 'border-brand text-brand bg-brand-light'
                    : 'border-gray-300 text-gray-400 bg-white'
                }`}
              >
                {done ? '✓' : i + 1}
              </div>
              <span className={`text-xs mt-1 text-center leading-tight max-w-[64px] ${active ? 'text-brand font-medium' : 'text-gray-400'}`}>
                {label}
              </span>
            </div>
            {i < steps.length - 1 && (
              <div className={`flex-1 h-0.5 mx-1 mb-5 ${done ? 'bg-brand' : 'bg-gray-200'}`} />
            )}
          </div>
        )
      })}
    </div>
  )
}
