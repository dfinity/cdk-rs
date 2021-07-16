export const idlFactory = ({ IDL }) => {
  return IDL.Service({
    'aiMove' : IDL.Func([IDL.Text], [], []),
    'getFen' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Text)], ['query']),
    'move' : IDL.Func([IDL.Text, IDL.Text], [IDL.Bool], []),
    'new' : IDL.Func([IDL.Text, IDL.Bool], [], []),
  });
};
export const init = ({ IDL }) => { return []; };