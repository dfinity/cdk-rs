import type { Principal } from '@dfinity/principal';
export interface _SERVICE {
  'aiMove' : (arg_0: string) => Promise<undefined>,
  'getFen' : (arg_0: string) => Promise<[] | [string]>,
  'move' : (arg_0: string, arg_1: string) => Promise<boolean>,
  'new' : (arg_0: string, arg_1: boolean) => Promise<undefined>,
}