@{
    IncludeRules = @(
        'PSAvoidUsingEmptyCatchBlock',
        'PSAvoidUsingPositionalParameters',
        'PSAvoidUsingPlainTextForPassword'
    )
    ExcludeRules = @(
        'PSAvoidUsingWriteHost',
        'PSUseDeclaredVarsMoreThanAssignments',
        'PSAvoidTrailingWhitespace',
        'PSUseUTF8EncodingForHelpFile',
        'PSMissingModuleManifestField',
        'PSAvoidGlobalVars',
        'PSUseCompatibleCommands',
        'PSUseCompatibleTypes'
    )
}
