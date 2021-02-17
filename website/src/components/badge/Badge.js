import { React } from 'react'
import './Badge.css'
import '../../index.css'

export const Badge = (args) => {
  return (
    <div key={args.i} className="box">
      <img
        alt="Badge"
        src={args.src}
        style={{
          width: args.size,
        }}
      />
      <div className="badge-info">{args.placeholder}</div>
    </div>
  )
}
