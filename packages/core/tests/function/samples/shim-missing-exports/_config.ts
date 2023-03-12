import { defineTestConfig } from '@/utils'
import * as t from 'vitest'

export default defineTestConfig({
  options: {
    shimMissingExports: true,
  },
  exports(exports) {
    t.expect(exports).toEqual({
      missingDefault: undefined,
      missingNamed: undefined,
    })
  },
})
